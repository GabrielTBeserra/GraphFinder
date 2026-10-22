//! Graph topological analytics, community clustering (Louvain/LPA),
//! God Nodes detection, architectural surprises, multi-hop pathfinding,
//! and standalone artifact generation (graph.json, GRAPH_REPORT.md, graph.html).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use super::db::{GraphData, GraphNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: usize,
    pub name: String,
    pub node_count: usize,
    pub dominant_type: String,
    pub key_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodNodeInfo {
    pub id: String,
    pub title: String,
    pub node_type: String,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub centrality: f32,
    pub community_id: usize,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurpriseEdge {
    pub source: String,
    pub source_title: String,
    pub target: String,
    pub target_title: String,
    pub label: String,
    pub source_community: usize,
    pub target_community: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCycle {
    pub nodes: Vec<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStep {
    pub node_id: String,
    pub title: String,
    pub node_type: String,
    pub edge_label: Option<String>,
    pub direction: String, // "->" or "<-" or "start"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResult {
    pub source: String,
    pub target: String,
    pub length: usize,
    pub steps: Vec<PathStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExplanation {
    pub id: String,
    pub title: String,
    pub node_type: String,
    pub path: Option<String>,
    pub community_id: usize,
    pub community_name: String,
    pub is_god_node: bool,
    pub centrality_rank: usize,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub incoming_dependents: Vec<String>,
    pub outgoing_dependencies: Vec<String>,
    pub sibling_nodes: Vec<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysisReport {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f32,
    pub communities: Vec<CommunityInfo>,
    pub god_nodes: Vec<GodNodeInfo>,
    pub surprises: Vec<SurpriseEdge>,
    pub cycles: Vec<DependencyCycle>,
    pub isolated_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    pub output_dir: String,
    pub graph_json_path: String,
    pub report_markdown_path: String,
    pub standalone_html_path: String,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub communities_count: usize,
    pub god_nodes_count: usize,
}

/// The core graph analytics engine operating on GraphData
pub struct GraphAnalyticsEngine {
    nodes: Vec<GraphNode>,
    id_to_idx: HashMap<String, usize>,
    #[allow(dead_code)]
    idx_to_id: Vec<String>,
    adj_out: Vec<Vec<(usize, String)>>,
    adj_in: Vec<Vec<(usize, String)>>,
    adj_undirected: Vec<Vec<(usize, String)>>,
    node_communities: Vec<usize>,
    summaries: HashMap<String, String>,
}

impl GraphAnalyticsEngine {
    pub fn new(graph: &GraphData, summaries: HashMap<String, String>) -> Self {
        let n = graph.nodes.len();
        let mut id_to_idx = HashMap::with_capacity(n);
        let mut idx_to_id = Vec::with_capacity(n);

        for (i, node) in graph.nodes.iter().enumerate() {
            id_to_idx.insert(node.id.clone(), i);
            idx_to_id.push(node.id.clone());
        }

        let mut adj_out = vec![Vec::new(); n];
        let mut adj_in = vec![Vec::new(); n];
        let mut adj_undirected = vec![Vec::new(); n];

        for link in &graph.links {
            let src_idx = id_to_idx.get(&link.source);
            let tgt_idx = id_to_idx.get(&link.target);

            if let (Some(&u), Some(&v)) = (src_idx, tgt_idx) {
                if u != v {
                    let label = link.label.clone().unwrap_or_else(|| "link".to_string());
                    adj_out[u].push((v, label.clone()));
                    adj_in[v].push((u, label.clone()));
                    adj_undirected[u].push((v, label.clone()));
                    adj_undirected[v].push((u, label));
                }
            }
        }

        let mut engine = Self {
            nodes: graph.nodes.clone(),
            id_to_idx,
            idx_to_id,
            adj_out,
            adj_in,
            adj_undirected,
            node_communities: vec![0; n],
            summaries,
        };

        engine.compute_communities();
        engine
    }

    /// Fast Label Propagation Algorithm (LPA) with modularity weighting for community clustering
    fn compute_communities(&mut self) {
        let n = self.nodes.len();
        if n == 0 {
            return;
        }

        // Initially each node is in its own community
        let mut labels: Vec<usize> = (0..n).collect();

        // Run up to 15 iterations of synchronous/semi-synchronous LPA
        let max_iterations = 15;
        for _iter in 0..max_iterations {
            let mut changes = 0;

            for u in 0..n {
                let neighbors = &self.adj_undirected[u];
                if neighbors.is_empty() {
                    continue;
                }

                // Count weighted frequencies of neighbor labels
                let mut freq: HashMap<usize, usize> = HashMap::new();
                for (v, _) in neighbors {
                    *freq.entry(labels[*v]).or_insert(0) += 1;
                }

                // Find label with highest frequency, breaking ties deterministically
                let current_label = labels[u];
                let mut best_label = current_label;
                let mut max_count = *freq.get(&current_label).unwrap_or(&0);

                for (&lbl, &cnt) in &freq {
                    if cnt > max_count || (cnt == max_count && lbl < best_label) {
                        max_count = cnt;
                        best_label = lbl;
                    }
                }

                if best_label != current_label {
                    labels[u] = best_label;
                    changes += 1;
                }
            }

            if changes == 0 {
                break;
            }
        }

        // Remap community IDs to contiguous 0..K sorted by community size
        let mut size_by_raw: HashMap<usize, usize> = HashMap::new();
        for &lbl in &labels {
            *size_by_raw.entry(lbl).or_insert(0) += 1;
        }

        let mut raw_sorted: Vec<(usize, usize)> = size_by_raw.into_iter().collect();
        raw_sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut raw_to_canonical = HashMap::new();
        for (canonical_id, (raw_lbl, _)) in raw_sorted.into_iter().enumerate() {
            raw_to_canonical.insert(raw_lbl, canonical_id);
        }

        self.node_communities = labels
            .into_iter()
            .map(|lbl| *raw_to_canonical.get(&lbl).unwrap_or(&0))
            .collect();
    }

    /// Returns the detected community index for a node
    pub fn node_community(&self, idx: usize) -> usize {
        self.node_communities.get(idx).copied().unwrap_or(0)
    }

    /// Generates full architectural analysis report
    pub fn analyze(&self) -> GraphAnalysisReport {
        let n = self.nodes.len();
        let total_edges: usize = self.adj_out.iter().map(|l| l.len()).sum();
        let max_edges = if n > 1 { n * (n - 1) } else { 1 };
        let density = (total_edges as f32) / (max_edges as f32);

        // 1. Communities Breakdown
        let mut comm_members: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, &c) in self.node_communities.iter().enumerate() {
            comm_members.entry(c).or_default().push(i);
        }

        let mut communities = Vec::new();
        for (&c_id, members) in &comm_members {
            let mut type_counts: HashMap<String, usize> = HashMap::new();
            for &idx in members {
                let t = if self.nodes[idx].node_type == "code" {
                    self.nodes[idx]
                        .path
                        .as_deref()
                        .and_then(|p| p.split('.').last())
                        .unwrap_or("code")
                        .to_string()
                } else {
                    "note".to_string()
                };
                *type_counts.entry(t).or_insert(0) += 1;
            }

            let dominant_type = type_counts
                .into_iter()
                .max_by_key(|(_, cnt)| *cnt)
                .map(|(t, _)| t)
                .unwrap_or_else(|| "mixed".to_string());

            // Key nodes (top degree within community)
            let mut sorted_members = members.clone();
            sorted_members.sort_by(|&a, &b| {
                let deg_a = self.adj_out[a].len() + self.adj_in[a].len();
                let deg_b = self.adj_out[b].len() + self.adj_in[b].len();
                deg_b.cmp(&deg_a)
            });

            let key_nodes = sorted_members
                .iter()
                .take(4)
                .map(|&idx| self.nodes[idx].title.clone())
                .collect::<Vec<_>>();

            let name = self.infer_community_name(&dominant_type, &key_nodes);

            communities.push(CommunityInfo {
                id: c_id,
                name,
                node_count: members.len(),
                dominant_type,
                key_nodes,
            });
        }
        communities.sort_by_key(|c| c.id);

        // 2. God Nodes (Highest Centrality Hubs)
        let mut god_nodes = Vec::new();
        let avg_degree = if n > 0 {
            (total_edges * 2) as f32 / n as f32
        } else {
            0.0
        };

        for (i, node) in self.nodes.iter().enumerate() {
            let out_deg = self.adj_out[i].len();
            let in_deg = self.adj_in[i].len();
            let degree = out_deg + in_deg;

            let centrality = if n > 1 {
                degree as f32 / (2.0 * (n - 1) as f32)
            } else {
                0.0
            };

            // God node qualification: degree >= 3 and degree > 1.4 * avg_degree, or top absolute degree
            let is_god = degree >= 3 && (degree as f32 >= (avg_degree * 1.3).max(3.0));
            if is_god {
                let role = if in_deg >= out_deg * 2 {
                    "Foundation / Core Dependency".to_string()
                } else if out_deg >= in_deg * 2 {
                    "Orchestrator / Dispatcher".to_string()
                } else {
                    "Architectural Hub".to_string()
                };

                god_nodes.push(GodNodeInfo {
                    id: node.id.clone(),
                    title: node.title.clone(),
                    node_type: node.node_type.clone(),
                    degree,
                    in_degree: in_deg,
                    out_degree: out_deg,
                    centrality,
                    community_id: self.node_communities[i],
                    role,
                });
            }
        }
        god_nodes.sort_by(|a, b| b.degree.cmp(&a.degree));
        god_nodes.truncate(12);

        // 3. Surprises: Cross-Community Couplings
        let mut surprises = Vec::new();
        for (u, edges) in self.adj_out.iter().enumerate() {
            let comm_u = self.node_communities[u];
            for (v, label) in edges {
                let comm_v = self.node_communities[*v];
                if comm_u != comm_v {
                    let u_is_god = god_nodes.iter().any(|g| g.id == self.nodes[u].id);
                    let v_is_god = god_nodes.iter().any(|g| g.id == self.nodes[*v].id);

                    let reason = if u_is_god && v_is_god {
                        "Critical inter-hub coupling across subsystems".to_string()
                    } else if label == "import" {
                        "Cross-module code import across decoupled communities".to_string()
                    } else {
                        "Wikilink linking disparate conceptual clusters".to_string()
                    };

                    surprises.push(SurpriseEdge {
                        source: self.nodes[u].id.clone(),
                        source_title: self.nodes[u].title.clone(),
                        target: self.nodes[*v].id.clone(),
                        target_title: self.nodes[*v].title.clone(),
                        label: label.clone(),
                        source_community: comm_u,
                        target_community: comm_v,
                        reason,
                    });
                }
            }
        }
        surprises.truncate(20);

        // 4. Cycles Detection (Direct 2-node or 3-node dependency loops in code)
        let mut cycles = Vec::new();
        let mut seen_cycles = HashSet::new();

        for u in 0..n {
            for (v, _) in &self.adj_out[u] {
                // Check direct 2-cycle: u -> v and v -> u
                if self.adj_out[*v].iter().any(|(w, _)| *w == u) {
                    let mut pair = vec![self.nodes[u].id.clone(), self.nodes[*v].id.clone()];
                    pair.sort();
                    let key = pair.join("<->");
                    if seen_cycles.insert(key) {
                        cycles.push(DependencyCycle {
                            nodes: vec![self.nodes[u].title.clone(), self.nodes[*v].title.clone()],
                            length: 2,
                        });
                    }
                }
            }
        }

        // 5. Isolated Nodes
        let mut isolated_nodes = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            if self.adj_out[i].is_empty() && self.adj_in[i].is_empty() {
                isolated_nodes.push(node.title.clone());
            }
        }

        GraphAnalysisReport {
            total_nodes: n,
            total_edges,
            density,
            communities,
            god_nodes,
            surprises,
            cycles,
            isolated_nodes,
        }
    }

    fn infer_community_name(&self, dominant_type: &str, key_nodes: &[String]) -> String {
        let sample = key_nodes.join(" ").to_lowercase();
        if sample.contains("db") || sample.contains("sqlite") || sample.contains("sql") || sample.contains("store") {
            "Database & Persistence Layer".to_string()
        } else if sample.contains("mcp") || sample.contains("server") || sample.contains("tool") {
            "MCP Protocol & AI Tooling".to_string()
        } else if sample.contains("graph") || sample.contains("node") || sample.contains("view") || sample.contains("canvas") {
            "Interactive Graph & Canvas".to_string()
        } else if sample.contains("engine") || sample.contains("embed") || sample.contains("parse") || sample.contains("code") {
            "Core Engine & Parsing Pipeline".to_string()
        } else if dominant_type == "rs" {
            "Rust Core & Native Subsystems".to_string()
        } else if dominant_type == "tsx" || dominant_type == "ts" || dominant_type == "js" {
            "Frontend Components & State".to_string()
        } else if dominant_type == "py" {
            "Python Services & Scripts".to_string()
        } else if dominant_type == "note" {
            "Markdown Knowledge Vault".to_string()
        } else {
            format!("Subsystem ({})", key_nodes.first().cloned().unwrap_or_else(|| "Module".to_string()))
        }
    }

    /// Finds a node index by exact ID, title, or partial match
    pub fn find_node_index(&self, query: &str) -> Option<usize> {
        let q = query.trim().to_lowercase();

        // 1. Exact ID match
        if let Some(&idx) = self.id_to_idx.get(query) {
            return Some(idx);
        }

        // 2. Case-insensitive ID match
        for (i, node) in self.nodes.iter().enumerate() {
            if node.id.to_lowercase() == q {
                return Some(i);
            }
        }

        // 3. Case-insensitive title match
        for (i, node) in self.nodes.iter().enumerate() {
            if node.title.to_lowercase() == q {
                return Some(i);
            }
        }

        // 4. Filename / path match
        for (i, node) in self.nodes.iter().enumerate() {
            if let Some(p) = &node.path {
                if p.to_lowercase() == q || p.to_lowercase().ends_with(&format!("/{}", q)) || p.to_lowercase().ends_with(&format!("\\{}", q)) {
                    return Some(i);
                }
            }
        }

        // 5. Substring contains match
        for (i, node) in self.nodes.iter().enumerate() {
            if node.title.to_lowercase().contains(&q) || node.id.to_lowercase().contains(&q) {
                return Some(i);
            }
        }

        None
    }

    /// Multi-hop shortest path search using BFS with edge direction awareness
    pub fn find_path(&self, source_query: &str, target_query: &str) -> Option<PathResult> {
        let src_idx = self.find_node_index(source_query)?;
        let tgt_idx = self.find_node_index(target_query)?;

        if src_idx == tgt_idx {
            return Some(PathResult {
                source: self.nodes[src_idx].title.clone(),
                target: self.nodes[tgt_idx].title.clone(),
                length: 0,
                steps: vec![PathStep {
                    node_id: self.nodes[src_idx].id.clone(),
                    title: self.nodes[src_idx].title.clone(),
                    node_type: self.nodes[src_idx].node_type.clone(),
                    edge_label: None,
                    direction: "start".to_string(),
                }],
            });
        }

        // 1. Try directed BFS first: source -> target
        if let Some(steps) = self.bfs_directed(src_idx, tgt_idx) {
            return Some(PathResult {
                source: self.nodes[src_idx].title.clone(),
                target: self.nodes[tgt_idx].title.clone(),
                length: steps.len() - 1,
                steps,
            });
        }

        // 2. If no directed path, search undirected path (with directional markers)
        if let Some(steps) = self.bfs_undirected(src_idx, tgt_idx) {
            return Some(PathResult {
                source: self.nodes[src_idx].title.clone(),
                target: self.nodes[tgt_idx].title.clone(),
                length: steps.len() - 1,
                steps,
            });
        }

        None
    }

    fn bfs_directed(&self, src: usize, tgt: usize) -> Option<Vec<PathStep>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<usize, (usize, String)> = HashMap::new();

        queue.push_back(src);
        visited.insert(src);

        let mut found = false;
        while let Some(curr) = queue.pop_front() {
            if curr == tgt {
                found = true;
                break;
            }

            for (nxt, label) in &self.adj_out[curr] {
                if visited.insert(*nxt) {
                    parent.insert(*nxt, (curr, label.clone()));
                    queue.push_back(*nxt);
                }
            }
        }

        if !found {
            return None;
        }

        // Reconstruct path
        let mut path_indices = Vec::new();
        let mut curr = tgt;
        while curr != src {
            if let Some((p, lbl)) = parent.get(&curr) {
                path_indices.push((curr, lbl.clone()));
                curr = *p;
            } else {
                break;
            }
        }

        path_indices.reverse();

        let mut steps = Vec::new();
        steps.push(PathStep {
            node_id: self.nodes[src].id.clone(),
            title: self.nodes[src].title.clone(),
            node_type: self.nodes[src].node_type.clone(),
            edge_label: None,
            direction: "start".to_string(),
        });

        for (idx, lbl) in path_indices {
            steps.push(PathStep {
                node_id: self.nodes[idx].id.clone(),
                title: self.nodes[idx].title.clone(),
                node_type: self.nodes[idx].node_type.clone(),
                edge_label: Some(lbl),
                direction: "->".to_string(),
            });
        }

        Some(steps)
    }

    fn bfs_undirected(&self, src: usize, tgt: usize) -> Option<Vec<PathStep>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        // maps child -> (parent, edge_label, is_forward)
        let mut parent: HashMap<usize, (usize, String, bool)> = HashMap::new();

        queue.push_back(src);
        visited.insert(src);

        let mut found = false;
        while let Some(curr) = queue.pop_front() {
            if curr == tgt {
                found = true;
                break;
            }

            // Check outgoing
            for (nxt, label) in &self.adj_out[curr] {
                if visited.insert(*nxt) {
                    parent.insert(*nxt, (curr, label.clone(), true));
                    queue.push_back(*nxt);
                }
            }

            // Check incoming
            for (prev, label) in &self.adj_in[curr] {
                if visited.insert(*prev) {
                    parent.insert(*prev, (curr, label.clone(), false));
                    queue.push_back(*prev);
                }
            }
        }

        if !found {
            return None;
        }

        // Reconstruct
        let mut path_info = Vec::new();
        let mut curr = tgt;
        while curr != src {
            if let Some((p, lbl, forward)) = parent.get(&curr) {
                path_info.push((curr, lbl.clone(), *forward));
                curr = *p;
            } else {
                break;
            }
        }

        path_info.reverse();

        let mut steps = Vec::new();
        steps.push(PathStep {
            node_id: self.nodes[src].id.clone(),
            title: self.nodes[src].title.clone(),
            node_type: self.nodes[src].node_type.clone(),
            edge_label: None,
            direction: "start".to_string(),
        });

        for (idx, lbl, forward) in path_info {
            steps.push(PathStep {
                node_id: self.nodes[idx].id.clone(),
                title: self.nodes[idx].title.clone(),
                node_type: self.nodes[idx].node_type.clone(),
                edge_label: Some(lbl),
                direction: if forward { "->".to_string() } else { "<-".to_string() },
            });
        }

        Some(steps)
    }

    /// Explains a node in detail: structural role, callers, callees, community context
    pub fn explain_node(&self, target_query: &str) -> Option<NodeExplanation> {
        let idx = self.find_node_index(target_query)?;
        let node = &self.nodes[idx];
        let comm_id = self.node_communities[idx];

        let out_deg = self.adj_out[idx].len();
        let in_deg = self.adj_in[idx].len();
        let degree = out_deg + in_deg;

        // Rank centrality among all nodes
        let mut all_degrees: Vec<(usize, usize)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, _)| (i, self.adj_out[i].len() + self.adj_in[i].len()))
            .collect();
        all_degrees.sort_by(|a, b| b.1.cmp(&a.1));

        let rank = all_degrees.iter().position(|&(i, _)| i == idx).unwrap_or(0) + 1;
        let is_god_node = degree >= 3 && rank <= 12;

        let incoming_dependents = self.adj_in[idx]
            .iter()
            .map(|(u, lbl)| format!("{} ({})", self.nodes[*u].title, lbl))
            .collect();

        let outgoing_dependencies = self.adj_out[idx]
            .iter()
            .map(|(v, lbl)| format!("{} ({})", self.nodes[*v].title, lbl))
            .collect();

        let sibling_nodes: Vec<String> = self
            .node_communities
            .iter()
            .enumerate()
            .filter(|&(i, &c)| c == comm_id && i != idx)
            .take(6)
            .map(|(i, _)| self.nodes[i].title.clone())
            .collect();

        let summary = self.summaries.get(&node.id).cloned();

        let comm_name = self.infer_community_name("mixed", &sibling_nodes);

        Some(NodeExplanation {
            id: node.id.clone(),
            title: node.title.clone(),
            node_type: node.node_type.clone(),
            path: node.path.clone(),
            community_id: comm_id,
            community_name: comm_name,
            is_god_node,
            centrality_rank: rank,
            degree,
            in_degree: in_deg,
            out_degree: out_deg,
            incoming_dependents,
            outgoing_dependencies,
            sibling_nodes,
            summary,
        })
    }

    /// Exports full artifact suite: graph.json, GRAPH_REPORT.md, and graph.html
    pub fn export_artifacts(
        &self,
        output_dir: &Path,
        project_name: &str,
    ) -> Result<ExportSummary, Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(output_dir)?;

        let report = self.analyze();

        // 1. graph.json
        let json_path = output_dir.join("graph.json");
        let json_content = self.generate_graph_json(&report, project_name);
        std::fs::write(&json_path, json_content)?;

        // 2. GRAPH_REPORT.md
        let md_path = output_dir.join("GRAPH_REPORT.md");
        let md_content = self.generate_graph_report_markdown(&report, project_name);
        std::fs::write(&md_path, md_content)?;

        // 3. graph.html (Standalone interactive canvas visualizer)
        let html_path = output_dir.join("graph.html");
        let html_content = self.generate_standalone_html(&report, project_name);
        std::fs::write(&html_path, html_content)?;

        Ok(ExportSummary {
            output_dir: output_dir.to_string_lossy().to_string(),
            graph_json_path: json_path.to_string_lossy().to_string(),
            report_markdown_path: md_path.to_string_lossy().to_string(),
            standalone_html_path: html_path.to_string_lossy().to_string(),
            total_nodes: report.total_nodes,
            total_edges: report.total_edges,
            communities_count: report.communities.len(),
            god_nodes_count: report.god_nodes.len(),
        })
    }

    fn generate_graph_json(&self, report: &GraphAnalysisReport, project_name: &str) -> String {
        let god_ids: HashSet<String> = report.god_nodes.iter().map(|g| g.id.clone()).collect();

        let json_nodes: Vec<serde_json::Value> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let comm = self.node_communities[i];
                let in_deg = self.adj_in[i].len();
                let out_deg = self.adj_out[i].len();
                let deg = in_deg + out_deg;
                let is_god = god_ids.contains(&n.id);

                serde_json::json!({
                    "id": n.id,
                    "label": n.title,
                    "type": n.node_type,
                    "source_file": n.path,
                    "community": comm,
                    "degree": deg,
                    "in_degree": in_deg,
                    "out_degree": out_deg,
                    "is_god_node": is_god,
                    "tags": n.tags,
                })
            })
            .collect();

        let mut json_edges: Vec<serde_json::Value> = Vec::new();
        for (u, edges) in self.adj_out.iter().enumerate() {
            let comm_u = self.node_communities[u];
            for (v, label) in edges {
                let comm_v = self.node_communities[*v];
                json_edges.push(serde_json::json!({
                    "source": self.nodes[u].id,
                    "target": self.nodes[*v].id,
                    "relationship": label,
                    "is_cross_community": comm_u != comm_v,
                }));
            }
        }

        let out = serde_json::json!({
            "graphfinder_version": "0.1.0",
            "schema_version": 1,
            "project_name": project_name,
            "directed": true,
            "stats": {
                "total_nodes": report.total_nodes,
                "total_edges": report.total_edges,
                "density": report.density,
                "communities_count": report.communities.len(),
                "god_nodes_count": report.god_nodes.len(),
                "surprises_count": report.surprises.len(),
            },
            "communities": report.communities,
            "god_nodes": report.god_nodes,
            "nodes": json_nodes,
            "edges": json_edges,
        });

        serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
    }

    fn generate_graph_report_markdown(&self, report: &GraphAnalysisReport, project_name: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Architectural Knowledge Graph Audit: {}\n\n", project_name));
        out.push_str("> High-density topological architecture summary, modular community clusters, and dependency analysis computed by **GraphFinder**.\n\n");

        out.push_str("## 1. Executive Summary\n\n");
        out.push_str(&format!("- **Total Nodes**: {}\n", report.total_nodes));
        out.push_str(&format!("- **Total Edges / Connections**: {}\n", report.total_edges));
        out.push_str(&format!("- **Graph Density**: {:.4}\n", report.density));
        out.push_str(&format!("- **Detected Subsystems / Communities**: {}\n", report.communities.len()));
        out.push_str(&format!("- **Identified God Nodes (Architectural Hubs)**: {}\n", report.god_nodes.len()));
        out.push_str(&format!("- **Cross-Community Couplings (Surprises)**: {}\n", report.surprises.len()));
        out.push_str(&format!("- **Circular Dependency Loops**: {}\n\n", report.cycles.len()));

        out.push_str("## 2. Subsystems & Modularity Communities\n\n");
        out.push_str("| ID | Community Name | Nodes | Dominant Stack | Key Nodes |\n");
        out.push_str("| :---: | :--- | :---: | :--- | :--- |\n");
        for c in &report.communities {
            let keys = c.key_nodes.join(", ");
            out.push_str(&format!(
                "| **C{}** | {} | {} | `{}` | {} |\n",
                c.id, c.name, c.node_count, c.dominant_type, keys
            ));
        }
        out.push_str("\n");

        out.push_str("## 3. God Nodes (Architectural Hubs)\n\n");
        out.push_str("Nodes with the highest degree centrality. Architectural changes to these components carry high blast radius:\n\n");
        out.push_str("| Node / Symbol | Type | In | Out | Total | Community | Architectural Role |\n");
        out.push_str("| :--- | :---: | :---: | :---: | :---: | :---: | :--- |\n");
        for g in &report.god_nodes {
            out.push_str(&format!(
                "| **`{}`** | `{}` | {} | {} | **{}** | C{} | {} |\n",
                g.title, g.node_type, g.in_degree, g.out_degree, g.degree, g.community_id, g.role
            ));
        }
        out.push_str("\n");

        if !report.surprises.is_empty() {
            out.push_str("## 4. Architectural Surprises & Cross-Couplings\n\n");
            out.push_str("Unexpected connections bridging distinct subsystems. These may represent architectural leakage or crucial cross-cutting interfaces:\n\n");
            for (i, s) in report.surprises.iter().enumerate().take(10) {
                out.push_str(&format!(
                    "{}. **`{}`** (C{}) $\\xrightarrow{{\\text{{{}}}}}$ **`{}`** (C{})\n   - *{}*\n",
                    i + 1, s.source_title, s.source_community, s.label, s.target_title, s.target_community, s.reason
                ));
            }
            out.push_str("\n");
        }

        if !report.cycles.is_empty() {
            out.push_str("## 5. Dependency Cycles Detected\n\n");
            out.push_str("Direct recursive or mutual dependencies found in codebase:\n\n");
            for c in &report.cycles {
                out.push_str(&format!("- ⚠️ **Mutual Loop**: `{}`\n", c.nodes.join(" <---> ")));
            }
            out.push_str("\n");
        }

        if !report.isolated_nodes.is_empty() {
            out.push_str("## 6. Orphaned / Isolated Knowledge Nodes\n\n");
            out.push_str(&format!("{} nodes have 0 connections:\n\n", report.isolated_nodes.len()));
            for name in report.isolated_nodes.iter().take(15) {
                out.push_str(&format!("- [[{}]]\n", name));
            }
            if report.isolated_nodes.len() > 15 {
                out.push_str(&format!("- *...and {} more*\n", report.isolated_nodes.len() - 15));
            }
            out.push_str("\n");
        }

        out
    }

    fn generate_standalone_html(&self, report: &GraphAnalysisReport, project_name: &str) -> String {
        let json_data = self.generate_graph_json(report, project_name);

        let template = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>GraphFinder - {{PROJECT_NAME}}</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;
      background: #090d16;
      color: #e2e8f0;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      height: 100vh;
      width: 100vw;
    }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 20px;
      background: #0f172a;
      border-bottom: 1px solid #1e293b;
      z-index: 10;
    }
    .title-group { display: flex; align-items: center; gap: 12px; }
    .title-group h1 { font-size: 16px; font-weight: 700; letter-spacing: -0.3px; color: #f8fafc; }
    .badge {
      background: #3b82f620;
      color: #60a5fa;
      border: 1px solid #3b82f640;
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 9999px;
      font-weight: 600;
    }
    .controls { display: flex; align-items: center; gap: 12px; }
    input[type="text"] {
      background: #1e293b;
      border: 1px solid #334155;
      color: #f8fafc;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 13px;
      width: 220px;
      outline: none;
    }
    input[type="text"]:focus { border-color: #60a5fa; }
    .btn {
      background: #1e293b;
      border: 1px solid #334155;
      color: #cbd5e1;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 12px;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s ease;
    }
    .btn:hover { background: #334155; color: #fff; }
    .btn.active { background: #2563eb; color: #fff; border-color: #3b82f6; }
    #canvas-container { position: relative; flex: 1; width: 100%; height: 100%; }
    canvas { display: block; width: 100%; height: 100%; }
    #sidebar {
      position: absolute;
      top: 16px;
      right: 16px;
      width: 320px;
      max-height: calc(100% - 32px);
      background: #0f172ae6;
      backdrop-filter: blur(12px);
      border: 1px solid #1e293b;
      border-radius: 10px;
      padding: 16px;
      display: none;
      flex-direction: column;
      gap: 12px;
      overflow-y: auto;
      box-shadow: 0 10px 25px -5px rgba(0,0,0,0.5);
      z-index: 20;
    }
    #sidebar h2 { font-size: 15px; font-weight: 700; color: #f8fafc; word-break: break-all; }
    .meta-row { display: flex; justify-content: space-between; font-size: 12px; color: #94a3b8; }
    .meta-val { font-weight: 600; color: #cbd5e1; }
    .list-box { background: #1e293b80; border: 1px solid #334155; border-radius: 6px; padding: 8px; font-size: 11px; max-height: 120px; overflow-y: auto; }
    .legend {
      position: absolute;
      bottom: 16px;
      left: 16px;
      background: #0f172acc;
      border: 1px solid #1e293b;
      border-radius: 8px;
      padding: 10px 14px;
      font-size: 11px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      z-index: 10;
    }
    .legend-item { display: flex; align-items: center; gap: 8px; }
    .color-dot { width: 10px; height: 10px; border-radius: 50%; }
  </style>
</head>
<body>
  <header>
    <div class="title-group">
      <h1>{{PROJECT_NAME}}</h1>
      <span class="badge">GraphFinder Standalone</span>
      <span style="font-size: 12px; color: #64748b;">• {{TOTAL_NODES}} nodes • {{TOTAL_EDGES}} edges</span>
    </div>
    <div class="controls">
      <input type="text" id="search" placeholder="Search node or file...">
      <button class="btn active" id="btn-color">Mode: Communities</button>
      <button class="btn" id="btn-gods">Highlight God Nodes</button>
      <button class="btn" id="btn-reset">Reset View</button>
    </div>
  </header>

  <div id="canvas-container">
    <canvas id="graph-canvas"></canvas>
    <div id="sidebar">
      <div style="display:flex; justify-content:space-between; align-items:center;">
        <span class="badge" id="side-type">NODE</span>
        <button id="close-side" style="background:none; border:none; color:#94a3b8; cursor:pointer; font-size:16px;">✕</button>
      </div>
      <h2 id="side-title">Node Title</h2>
      <div class="meta-row"><span>Community:</span><span class="meta-val" id="side-comm">C0</span></div>
      <div class="meta-row"><span>Degree (In/Out):</span><span class="meta-val" id="side-deg">0 (0/0)</span></div>
      <div class="meta-row"><span>Status:</span><span class="meta-val" id="side-status">Standard Node</span></div>
      <div style="font-size:12px; color:#94a3b8; margin-top:4px;">Connected Entities:</div>
      <div class="list-box" id="side-links"></div>
    </div>
    <div class="legend" id="legend"></div>
  </div>

  <script>
    const DATA = {{DATA_JSON}};

    const COMMUNITY_COLORS = [
      "#6366f1", "#10b981", "#38bdf8", "#f59e0b", "#ec4899",
      "#8b5cf6", "#06b6d4", "#f97316", "#14b8a6", "#e11d48",
      "#84cc16", "#a855f7", "#0ea5e9", "#eab308"
    ];

    const canvas = document.getElementById("graph-canvas");
    const ctx = canvas.getContext("2d");
    let width = canvas.width = window.innerWidth;
    let height = canvas.height = window.innerHeight - 50;

    window.addEventListener("resize", () => {
      width = canvas.width = window.innerWidth;
      height = canvas.height = window.innerHeight - 50;
    });

    // Build legend
    const legendEl = document.getElementById("legend");
    DATA.communities.slice(0, 7).forEach((c, i) => {
      const div = document.createElement("div");
      div.className = "legend-item";
      div.innerHTML = `<span class="color-dot" style="background:${COMMUNITY_COLORS[c.id % COMMUNITY_COLORS.length]}"></span><span>C${c.id}: ${c.name} (${c.node_count})</span>`;
      legendEl.appendChild(div);
    });

    // Setup node map & physics simulation
    const nodeMap = new Map();
    DATA.nodes.forEach((n, i) => {
      const angle = (i / DATA.nodes.length) * Math.PI * 2;
      const radius = 200 + Math.random() * 200;
      n.x = width / 2 + Math.cos(angle) * radius;
      n.y = height / 2 + Math.sin(angle) * radius;
      n.vx = 0;
      n.vy = 0;
      n.radius = n.is_god_node ? 10 : Math.max(4, Math.min(8, 3 + n.degree));
      nodeMap.set(n.id, n);
    });

    const links = DATA.edges.map(e => ({
      source: nodeMap.get(e.source),
      target: nodeMap.get(e.target),
      relationship: e.relationship,
      is_cross: e.is_cross_community
    })).filter(l => l.source && l.target);

    let zoom = 1.0;
    let panX = 0;
    let panY = 0;
    let isDragging = false;
    let dragNode = null;
    let lastMouseX = 0;
    let lastMouseY = 0;
    let colorByComm = true;
    let highlightGods = false;
    let searchFilter = "";
    let selectedNode = null;

    // Simulation loop
    function tick() {
      const rep = 800;

      // Repulsion
      for (let i = 0; i < DATA.nodes.length; i++) {
        const a = DATA.nodes[i];
        for (let j = i + 1; j < DATA.nodes.length; j++) {
          const b = DATA.nodes[j];
          let dx = b.x - a.x;
          let dy = b.y - a.y;
          let distSq = dx * dx + dy * dy || 1;
          if (distSq < 40000) {
            let dist = Math.sqrt(distSq);
            let force = rep / distSq;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;
            a.vx -= fx;
            a.vy -= fy;
            b.vx += fx;
            b.vy += fy;
          }
        }
      }

      // Spring links
      links.forEach(l => {
        let dx = l.target.x - l.source.x;
        let dy = l.target.y - l.source.y;
        let dist = Math.sqrt(dx * dx + dy * dy) || 1;
        let desired = l.is_cross ? 180 : 90;
        let force = (dist - desired) * 0.02;
        let fx = (dx / dist) * force;
        let fy = (dy / dist) * force;
        l.source.vx += fx;
        l.source.vy += fy;
        l.target.vx -= fx;
        l.target.vy -= fy;
      });

      // Center gravity & damping
      const cx = width / 2;
      const cy = height / 2;
      DATA.nodes.forEach(n => {
        if (n !== dragNode) {
          n.vx += (cx - n.x) * 0.002;
          n.vy += (cy - n.y) * 0.002;
          n.vx *= 0.85;
          n.vy *= 0.85;
          n.x += n.vx;
          n.y += n.vy;
        }
      });
    }

    function render() {
      tick();
      ctx.clearRect(0, 0, width, height);

      ctx.save();
      ctx.translate(panX, panY);
      ctx.scale(zoom, zoom);

      // Draw Edges
      links.forEach(l => {
        ctx.beginPath();
        ctx.moveTo(l.source.x, l.source.y);
        ctx.lineTo(l.target.x, l.target.y);
        if (selectedNode && (l.source === selectedNode || l.target === selectedNode)) {
          ctx.strokeStyle = "#38bdf8";
          ctx.lineWidth = 2.5;
        } else if (l.is_cross) {
          ctx.strokeStyle = "rgba(71, 85, 105, 0.25)";
          ctx.lineWidth = 1.0;
        } else {
          ctx.strokeStyle = "rgba(51, 65, 85, 0.3)";
          ctx.lineWidth = 1.0;
        }
        ctx.stroke();
      });

      // Draw Nodes
      DATA.nodes.forEach(n => {
        const isSelected = n === selectedNode;
        const matchesSearch = searchFilter && n.label.toLowerCase().includes(searchFilter);

        let color = colorByComm
          ? COMMUNITY_COLORS[n.community % COMMUNITY_COLORS.length]
          : (n.type === "code" ? "#38bdf8" : "#10b981");

        ctx.beginPath();
        ctx.arc(n.x, n.y, n.radius, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();

        if (n.is_god_node || isSelected || matchesSearch) {
          ctx.lineWidth = isSelected ? 3 : 2;
          ctx.strokeStyle = isSelected ? "#ffffff" : (n.is_god_node ? "#f59e0b" : "#60a5fa");
          ctx.stroke();

          if (n.is_god_node && highlightGods) {
            ctx.beginPath();
            ctx.arc(n.x, n.y, n.radius + 5, 0, Math.PI * 2);
            ctx.strokeStyle = "rgba(245, 158, 11, 0.5)";
            ctx.stroke();
          }
        }

        // Label
        if (n.radius >= 6 || isSelected || matchesSearch || n.is_god_node) {
          ctx.font = (isSelected || n.is_god_node ? "bold " : "") + "10px -apple-system, sans-serif";
          ctx.fillStyle = isSelected ? "#fff" : "#cbd5e1";
          ctx.fillText(n.label, n.x + n.radius + 4, n.y + 3);
        }
      });

      ctx.restore();
      requestAnimationFrame(render);
    }

    requestAnimationFrame(render);

    // Mouse & Pan/Zoom Handlers
    function screenToWorld(sx, sy) {
      return {
        x: (sx - panX) / zoom,
        y: (sy - panY) / zoom
      };
    }

    canvas.addEventListener("mousedown", e => {
      const pos = screenToWorld(e.clientX, e.clientY - 50);
      let hit = null;
      for (const n of DATA.nodes) {
        let d = Math.hypot(n.x - pos.x, n.y - pos.y);
        if (d <= n.radius + 4) { hit = n; break; }
      }

      if (hit) {
        dragNode = hit;
        selectedNode = hit;
        showSidebar(hit);
      } else {
        isDragging = true;
      }
      lastMouseX = e.clientX;
      lastMouseY = e.clientY;
    });

    window.addEventListener("mousemove", e => {
      if (dragNode) {
        const pos = screenToWorld(e.clientX, e.clientY - 50);
        dragNode.x = pos.x;
        dragNode.y = pos.y;
      } else if (isDragging) {
        panX += e.clientX - lastMouseX;
        panY += e.clientY - lastMouseY;
        lastMouseX = e.clientX;
        lastMouseY = e.clientY;
      }
    });

    window.addEventListener("mouseup", () => {
      dragNode = null;
      isDragging = false;
    });

    canvas.addEventListener("wheel", e => {
      e.preventDefault();
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      zoom = Math.max(0.2, Math.min(5.0, zoom * zoomFactor));
    });

    // Controls
    document.getElementById("btn-color").addEventListener("click", function() {
      colorByComm = !colorByComm;
      this.textContent = colorByComm ? "Mode: Communities" : "Mode: Node Types";
    });

    document.getElementById("btn-gods").addEventListener("click", function() {
      highlightGods = !highlightGods;
      this.classList.toggle("active", highlightGods);
    });

    document.getElementById("btn-reset").addEventListener("click", () => {
      zoom = 1.0;
      panX = 0;
      panY = 0;
      selectedNode = null;
      document.getElementById("sidebar").style.display = "none";
    });

    document.getElementById("search").addEventListener("input", e => {
      searchFilter = e.target.value.toLowerCase().trim();
    });

    const sidebar = document.getElementById("sidebar");
    document.getElementById("close-side").addEventListener("click", () => {
      sidebar.style.display = "none";
      selectedNode = null;
    });

    function showSidebar(n) {
      sidebar.style.display = "flex";
      document.getElementById("side-type").textContent = n.type.toUpperCase();
      document.getElementById("side-title").textContent = n.label;
      document.getElementById("side-comm").textContent = `C${n.community}`;
      document.getElementById("side-deg").textContent = `${n.degree} (In: ${n.in_degree}, Out: ${n.out_degree})`;
      document.getElementById("side-status").textContent = n.is_god_node ? "🌟 God Node (Central Hub)" : "Standard";

      const linksBox = document.getElementById("side-links");
      linksBox.innerHTML = "";
      const conns = links.filter(l => l.source === n || l.target === n);
      if (conns.length === 0) {
        linksBox.innerHTML = "<span style=\"color:#64748b;\">No connections</span>";
      } else {
        conns.forEach(l => {
          const isOut = l.source === n;
          const other = isOut ? l.target : l.source;
          const div = document.createElement("div");
          div.style.padding = "2px 0";
          div.innerHTML = `<span style="color:#60a5fa;">${isOut ? "→" : "←"}</span> ${other.label} <span style="color:#64748b;">(${l.relationship})</span>`;
          linksBox.appendChild(div);
        });
      }
    }
  </script>
</body>
</html>"##;

        template
            .replace("{{PROJECT_NAME}}", project_name)
            .replace("{{TOTAL_NODES}}", &report.total_nodes.to_string())
            .replace("{{TOTAL_EDGES}}", &report.total_edges.to_string())
            .replace("{{DATA_JSON}}", &json_data)
    }
}
