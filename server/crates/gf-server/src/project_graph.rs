use std::collections::HashMap;

use gf_core::graph_build::{analyze_graph, compute_communities, infer_edges, GraphDocument};
use gf_storage::repositories::DocumentGraphRow;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct GraphNodeResponse {
    pub id: Uuid,
    pub path: String,
    pub title: Option<String>,
    pub document_type: String,
    pub language: Option<String>,
    pub size_bytes: i64,
    pub degree: usize,
    pub community: usize,
    pub is_god_node: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdgeResponse {
    pub source: Uuid,
    pub target: Uuid,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GodNodeResponse {
    pub id: Uuid,
    pub path: String,
    pub title: Option<String>,
    pub degree: usize,
    pub centrality: f32,
    pub community: usize,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CycleResponse {
    pub nodes: Vec<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphAnalyticsResponse {
    pub god_nodes: Vec<GodNodeResponse>,
    pub cycles: Vec<CycleResponse>,
    pub cross_community_edges: usize,
    pub isolated_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectGraphResponse {
    pub project_id: Uuid,
    pub generation: i64,
    pub nodes: Vec<GraphNodeResponse>,
    pub edges: Vec<GraphEdgeResponse>,
    pub analytics: GraphAnalyticsResponse,
    pub truncated: bool,
    pub total_documents: usize,
    pub total_edges: usize,
    pub community_count: usize,
    /// "table" when edges came from first-class links rows persisted at index
    /// time, "inferred" when resolved on the fly from document metadata.
    pub edge_source: String,
}

struct RowDocument<'a> {
    row: &'a DocumentGraphRow,
}

impl GraphDocument for RowDocument<'_> {
    fn id_key(&self) -> &str {
        &self.row.path
    }
    fn path(&self) -> &str {
        &self.row.path
    }
    fn title(&self) -> Option<&str> {
        self.row.title.as_deref()
    }
    fn is_markdown(&self) -> bool {
        self.row.document_type == "markdown"
    }
    fn links(&self) -> Vec<String> {
        json_strings(&self.row.links)
    }
    fn imports(&self) -> Vec<String> {
        json_strings(&self.row.imports)
    }
}

fn json_strings(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Query-time inference fallback for generations indexed before the `links`
/// table was populated at index time.
pub fn infer_edges_for_rows(documents: &[DocumentGraphRow]) -> Vec<(String, String, String)> {
    let rows: Vec<RowDocument> = documents.iter().map(|row| RowDocument { row }).collect();
    infer_edges(&rows)
        .into_iter()
        .map(|edge| (edge.source, edge.target, edge.kind))
        .collect()
}

/// Builds the document-level knowledge/dependency graph for one generation
/// from pre-resolved edges (path-keyed), adding LPA communities plus the same
/// architectural analytics the desktop engine computes: god-node hubs,
/// dependency cycles, cross-community coupling and isolation counts.
pub fn build_project_graph(
    documents: Vec<DocumentGraphRow>,
    resolved_edges: Vec<(String, String, String)>,
    project_id: Uuid,
    generation: i64,
    limit: usize,
    edge_source: &str,
) -> ProjectGraphResponse {
    let total_documents = documents.len();

    let node_keys: Vec<String> = documents.iter().map(|row| row.path.clone()).collect();
    let resolved: Vec<gf_core::graph_build::ResolvedEdge> = resolved_edges
        .into_iter()
        .map(|(source, target, kind)| gf_core::graph_build::ResolvedEdge {
            source,
            target,
            kind,
        })
        .collect();
    let communities = compute_communities(&node_keys, &resolved);
    let community_count = communities.values().max().map(|max| max + 1).unwrap_or(0);
    let analytics = analyze_graph(&node_keys, &resolved, &communities);

    let id_by_path: HashMap<&str, Uuid> = documents
        .iter()
        .map(|row| (row.path.as_str(), row.id))
        .collect();

    let mut degree: HashMap<&str, usize> = HashMap::new();
    let mut edges: Vec<GraphEdgeResponse> = Vec::new();
    for edge in &resolved {
        let (Some(&source_id), Some(&target_id)) = (
            id_by_path.get(edge.source.as_str()),
            id_by_path.get(edge.target.as_str()),
        ) else {
            continue;
        };
        edges.push(GraphEdgeResponse {
            source: source_id,
            target: target_id,
            kind: edge.kind.clone(),
        });
        *degree.entry(edge.source.as_str()).or_insert(0) += 1;
        *degree.entry(edge.target.as_str()).or_insert(0) += 1;
    }
    let total_edges = edges.len();

    // Truncation keeps the most connected documents — the topology that
    // actually matters for visualization.
    let truncated = total_documents > limit;
    let kept: std::collections::HashSet<Uuid> = if truncated {
        let mut ranked: Vec<&DocumentGraphRow> = documents.iter().collect();
        ranked.sort_by(|a, b| {
            let da = degree.get(a.path.as_str()).copied().unwrap_or(0);
            let db = degree.get(b.path.as_str()).copied().unwrap_or(0);
            db.cmp(&da).then_with(|| a.path.cmp(&b.path))
        });
        ranked.iter().take(limit).map(|row| row.id).collect()
    } else {
        documents.iter().map(|row| row.id).collect()
    };

    let god_keys: std::collections::HashSet<&str> = analytics
        .god_nodes
        .iter()
        .map(|god| god.key.as_str())
        .collect();

    let nodes: Vec<GraphNodeResponse> = documents
        .iter()
        .filter(|row| kept.contains(&row.id))
        .map(|row| GraphNodeResponse {
            id: row.id,
            path: row.path.clone(),
            title: row.title.clone(),
            document_type: row.document_type.clone(),
            language: row.language.clone(),
            size_bytes: row.size_bytes,
            degree: degree.get(row.path.as_str()).copied().unwrap_or(0),
            community: communities.get(&row.path).copied().unwrap_or(0),
            is_god_node: god_keys.contains(row.path.as_str()),
            tags: json_strings(&row.tags),
        })
        .collect();

    let title_by_path: HashMap<&str, Option<&str>> = documents
        .iter()
        .map(|row| (row.path.as_str(), row.title.as_deref()))
        .collect();

    let analytics_response = GraphAnalyticsResponse {
        god_nodes: analytics
            .god_nodes
            .iter()
            .filter_map(|god| {
                let id = id_by_path.get(god.key.as_str())?;
                Some(GodNodeResponse {
                    id: *id,
                    path: god.key.clone(),
                    title: title_by_path
                        .get(god.key.as_str())
                        .copied()
                        .flatten()
                        .map(|t| t.to_string()),
                    degree: god.degree,
                    centrality: god.centrality,
                    community: god.community,
                    role: god.role.clone(),
                })
            })
            .collect(),
        cycles: analytics
            .cycles
            .iter()
            .take(20)
            .map(|cycle| CycleResponse {
                nodes: cycle.nodes.clone(),
                length: cycle.length,
            })
            .collect(),
        cross_community_edges: analytics.cross_community_edges,
        isolated_count: analytics.isolated_count,
    };

    let edges: Vec<GraphEdgeResponse> = edges
        .into_iter()
        .filter(|edge| kept.contains(&edge.source) && kept.contains(&edge.target))
        .collect();

    ProjectGraphResponse {
        project_id,
        generation,
        nodes,
        edges,
        analytics: analytics_response,
        truncated,
        total_documents,
        total_edges,
        community_count,
        edge_source: edge_source.to_string(),
    }
}
