//! Shared document-graph construction: high-quality wikilink/import
//! resolution plus topological analytics (LPA communities, centrality).
//!
//! This module is the single source of truth for turning indexed document
//! metadata into graph edges. Resolution is hashmap-accelerated (O(1) lookups)
//! while preserving the accuracy of per-file relative import resolution —
//! `./foo`, `../bar`, extension inference and `index` file resolution all
//! resolve against the importing document's location, exactly like the
//! language toolchains do.

use std::collections::{HashMap, HashSet};

/// Minimal document projection needed for graph construction, so the module
/// stays independent of any storage crate.
pub trait GraphDocument {
    fn id_key(&self) -> &str; // stable unique key (path or id string)
    fn path(&self) -> &str;
    fn title(&self) -> Option<&str>;
    fn is_markdown(&self) -> bool;
    fn links(&self) -> Vec<String>;
    fn imports(&self) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
}

fn normalize_key(value: &str) -> String {
    value
        .split('#')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_end_matches(".md")
        .to_ascii_lowercase()
        .replace(['-', '_'], " ")
}

fn file_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.split('.').next().unwrap_or(name).to_string()
}

fn module_specifier(import: &str) -> &str {
    let import = import.trim();
    if let Some(start) = import.find(['\'', '"']) {
        let quote = import.as_bytes()[start] as char;
        let rest = &import[start + 1..];
        if let Some(end) = rest.find(quote) {
            return &rest[..end];
        }
    }
    import
}

fn normalize_relative_path(base_path: &str, reference: &str) -> String {
    let mut segments = base_path
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or("")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    for segment in reference.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment.to_string()),
        }
    }
    segments.join("/")
}

fn module_path_candidates(path: &str) -> Vec<String> {
    let mut candidates = vec![path.to_string()];
    if let Some(stem) = path.strip_suffix(".js") {
        candidates.push(format!("{stem}.ts"));
        candidates.push(format!("{stem}.tsx"));
        candidates.push(format!("{stem}.jsx"));
    }
    if path.rsplit('/').next().is_some_and(|name| !name.contains('.')) {
        for extension in ["ts", "tsx", "js", "jsx", "rs", "py", "go"] {
            candidates.push(format!("{path}.{extension}"));
        }
        for extension in ["ts", "tsx", "js", "jsx"] {
            candidates.push(format!("{path}/index.{extension}"));
        }
    }
    candidates
}

/// Hashmap-accelerated resolution index over a document set.
pub struct ResolutionIndex {
    /// exact document path -> key
    by_path: HashMap<String, String>,
    /// normalized lookup key (path, stem or title) -> document key
    by_key: HashMap<String, String>,
    /// alphanumeric-squashed key -> document key (naming-drift fallback)
    by_squashed: HashMap<String, String>,
}

impl ResolutionIndex {
    pub fn build<D: GraphDocument>(documents: &[D]) -> Self {
        let mut by_path = HashMap::new();
        let mut by_key: HashMap<String, String> = HashMap::new();
        let mut by_squashed: HashMap<String, String> = HashMap::new();

        for doc in documents {
            by_path.insert(doc.path().to_string(), doc.id_key().to_string());
        }

        let mut insert_key = |key: String, id: &str| {
            if key.is_empty() {
                return;
            }
            by_key.entry(key.clone()).or_insert_with(|| id.to_string());
            let squashed: String = key.chars().filter(|c| c.is_alphanumeric()).collect();
            if !squashed.is_empty() {
                by_squashed.entry(squashed).or_insert_with(|| id.to_string());
            }
        };

        for doc in documents {
            let path = doc.path().replace('\\', "/");
            insert_key(normalize_key(&path), doc.id_key());
            insert_key(normalize_key(&file_stem(&path)), doc.id_key());
            // extensionless path — import specifiers commonly omit extensions
            let no_ext = path.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(&path);
            insert_key(normalize_key(no_ext), doc.id_key());
            if let Some(title) = doc.title() {
                insert_key(normalize_key(title), doc.id_key());
            }
            // path suffixes let partial references resolve ("foo/bar" inside
            // "src/foo/bar.ts"); shorter segments first so longer, more precise
            // suffixes inserted earlier win collisions deterministically.
            let segments: Vec<&str> = no_ext.split('/').collect();
            for start in 1..segments.len() {
                insert_key(normalize_key(&segments[start..].join("/")), doc.id_key());
            }
        }

        Self {
            by_path,
            by_key,
            by_squashed,
        }
    }

    /// Resolves a wikilink or bare reference to a document key.
    pub fn resolve_reference(&self, reference: &str) -> Option<String> {
        let reference = reference.trim();
        if reference.is_empty() {
            return None;
        }
        if let Some(key) = self.by_path.get(reference) {
            return Some(key.clone());
        }
        let normalized = normalize_key(reference);
        if let Some(key) = self.by_key.get(&normalized) {
            return Some(key.clone());
        }
        // last-resort match: "Some Note" ↔ "some-note.md" naming drift
        let squashed: String = normalized.chars().filter(|c| c.is_alphanumeric()).collect();
        if !squashed.is_empty() {
            if let Some(key) = self.by_squashed.get(&squashed) {
                return Some(key.clone());
            }
        }
        None
    }

    /// Resolves a code import exactly like language toolchains do: relative
    /// specifiers resolve against the importing file's directory with
    /// extension and index-file inference; bare specifiers fall back to
    /// reference resolution.
    pub fn resolve_import(&self, source_path: &str, import: &str) -> Option<String> {
        let reference = module_specifier(import);
        if reference.starts_with("./") || reference.starts_with("../") {
            let normalized = normalize_relative_path(source_path, reference);
            for candidate in module_path_candidates(&normalized) {
                if let Some(key) = self.by_path.get(&candidate) {
                    return Some(key.clone());
                }
            }
            return None;
        }
        // Rust-style `crate::foo::bar` / `use foo::bar` — try path form too
        let path_form = reference.replace("::", "/");
        let cleaned = path_form
            .trim_start_matches("crate/")
            .trim_start_matches("self/")
            .trim_start_matches("super/");
        if cleaned != reference {
            if let Some(key) = self.by_key.get(&normalize_key(cleaned)) {
                return Some(key.clone());
            }
        }
        self.resolve_reference(reference)
    }
}

/// Infers document-level edges from indexed metadata, with deterministic
/// dedupe. Markdown wikilinks become `link` edges, code imports become
/// `import` edges.
pub fn infer_edges<D: GraphDocument>(documents: &[D]) -> Vec<ResolvedEdge> {
    let index = ResolutionIndex::build(documents);
    let key_to_path: HashMap<&str, &str> = documents
        .iter()
        .map(|doc| (doc.id_key(), doc.path()))
        .collect();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();

    for doc in documents {
        let mut push = |target_key: String, kind: &str| {
            if target_key == doc.id_key() {
                return;
            }
            let Some(target_path) = key_to_path.get(target_key.as_str()) else {
                return;
            };
            let edge = ResolvedEdge {
                source: doc.path().to_string(),
                target: target_path.to_string(),
                kind: kind.to_string(),
            };
            if seen.insert(edge.clone()) {
                edges.push(edge);
            }
        };

        // Links are processed regardless of document type: in practice only
        // markdown carries wikilinks, but mixed vaults occasionally embed them
        // in other text formats, and the cost is a hashmap lookup per entry.
        for link in doc.links() {
            if let Some(target) = index.resolve_reference(&link) {
                push(target, "link");
            }
        }
        for import in doc.imports() {
            if let Some(target) = index.resolve_import(doc.path(), &import) {
                push(target, "import");
            }
        }
    }

    edges
}

/// Fast Label Propagation community detection with deterministic tie-breaking,
/// ported from the desktop engine so server and client agree on clustering.
/// Returns contiguous community ids ordered by community size (0 = largest).
pub fn compute_communities(node_keys: &[String], edges: &[ResolvedEdge]) -> HashMap<String, usize> {
    let n = node_keys.len();
    let mut result = HashMap::new();
    if n == 0 {
        return result;
    }

    let index_of: HashMap<&str, usize> = node_keys
        .iter()
        .enumerate()
        .map(|(i, key)| (key.as_str(), i))
        .collect();

    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    for edge in edges {
        // edges are keyed by path; node_keys are id_keys — caller passes paths
        // as node_keys, so direct lookup applies.
        if let (Some(&a), Some(&b)) = (index_of.get(edge.source.as_str()), index_of.get(edge.target.as_str())) {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }
    }

    let mut labels: Vec<usize> = (0..n).collect();
    for _ in 0..15 {
        let mut changes = 0;
        for u in 0..n {
            if adjacency[u].is_empty() {
                continue;
            }
            let mut freq: HashMap<usize, usize> = HashMap::new();
            for &v in &adjacency[u] {
                *freq.entry(labels[v]).or_insert(0) += 1;
            }
            let current = labels[u];
            let mut best = current;
            let mut max_count = *freq.get(&current).unwrap_or(&0);
            for (&label, &count) in &freq {
                if count > max_count || (count == max_count && label < best) {
                    max_count = count;
                    best = label;
                }
            }
            if best != current {
                labels[u] = best;
                changes += 1;
            }
        }
        if changes == 0 {
            break;
        }
    }

    // canonicalize: community 0 is the largest
    let mut sizes: HashMap<usize, usize> = HashMap::new();
    for &label in &labels {
        *sizes.entry(label).or_insert(0) += 1;
    }
    let mut ordered: Vec<(usize, usize)> = sizes.into_iter().collect();
    ordered.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let canonical: HashMap<usize, usize> = ordered
        .into_iter()
        .enumerate()
        .map(|(canonical, (raw, _))| (raw, canonical))
        .collect();

    for (i, key) in node_keys.iter().enumerate() {
        result.insert(key.clone(), *canonical.get(&labels[i]).unwrap_or(&0));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDoc {
        path: String,
        title: Option<String>,
        markdown: bool,
        links: Vec<String>,
        imports: Vec<String>,
    }

    impl GraphDocument for TestDoc {
        fn id_key(&self) -> &str {
            &self.path
        }
        fn path(&self) -> &str {
            &self.path
        }
        fn title(&self) -> Option<&str> {
            self.title.as_deref()
        }
        fn is_markdown(&self) -> bool {
            self.markdown
        }
        fn links(&self) -> Vec<String> {
            self.links.clone()
        }
        fn imports(&self) -> Vec<String> {
            self.imports.clone()
        }
    }

    fn doc(path: &str, markdown: bool, links: &[&str], imports: &[&str]) -> TestDoc {
        TestDoc {
            path: path.into(),
            title: None,
            markdown,
            links: links.iter().map(|s| s.to_string()).collect(),
            imports: imports.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn resolves_relative_imports_against_source_directory() {
        let docs = vec![
            doc("src/app/main.ts", false, &[], &["./utils/helpers", "../lib/core"]),
            doc("src/app/utils/helpers.ts", false, &[], &[]),
            doc("src/lib/core.ts", false, &[], &[]),
        ];
        let edges = infer_edges(&docs);
        assert!(edges.iter().any(|e| e.target == "src/app/utils/helpers.ts"));
        assert!(edges.iter().any(|e| e.target == "src/lib/core.ts"));
    }

    #[test]
    fn infers_extensionless_and_index_imports() {
        let docs = vec![
            doc("src/index.ts", false, &[], &["./components/Button"]),
            doc("src/components/Button/index.tsx", false, &[], &[]),
        ];
        let edges = infer_edges(&docs);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target, "src/components/Button/index.tsx");
    }

    #[test]
    fn resolves_wikilinks_with_naming_drift() {
        let mut target = doc("notes/Graph Model.md", true, &[], &[]);
        target.title = Some("Graph Model".into());
        let docs = vec![
            doc("notes/intro.md", true, &["Graph Model#Internals"], &[]),
            target,
        ];
        let edges = infer_edges(&docs);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].kind, "link");
    }

    #[test]
    fn analytics_detects_god_nodes_cycles_and_isolation() {
        let keys: Vec<String> = ["hub", "a", "b", "c", "lonely"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let edges = vec![
            ResolvedEdge { source: "a".into(), target: "hub".into(), kind: "import".into() },
            ResolvedEdge { source: "b".into(), target: "hub".into(), kind: "import".into() },
            ResolvedEdge { source: "c".into(), target: "hub".into(), kind: "import".into() },
            ResolvedEdge { source: "hub".into(), target: "a".into(), kind: "import".into() },
        ];
        let communities = compute_communities(&keys, &edges);
        let analytics = analyze_graph(&keys, &edges, &communities);

        assert_eq!(analytics.god_nodes.len(), 1);
        assert_eq!(analytics.god_nodes[0].key, "hub");
        assert_eq!(analytics.god_nodes[0].role, "Foundation / Core Dependency");

        // hub ↔ a is a directed 2-cycle
        assert!(analytics.cycles.iter().any(|cycle| cycle.length == 2));
        assert_eq!(analytics.isolated_count, 1);
    }

    #[test]
    fn communities_cluster_connected_components() {
        let keys: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let edges = vec![
            ResolvedEdge { source: "a".into(), target: "b".into(), kind: "link".into() },
            ResolvedEdge { source: "c".into(), target: "d".into(), kind: "link".into() },
        ];
        let communities = compute_communities(&keys, &edges);
        assert_eq!(communities["a"], communities["b"]);
        assert_eq!(communities["c"], communities["d"]);
        assert_ne!(communities["a"], communities["c"]);
    }
}

// ---------------------------------------------------------------------------
// Topological analytics — ported from the desktop engine so the server panel
// exposes the same architectural insight: god-node hubs, dependency cycles,
// cross-community coupling and isolated nodes.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GodNode {
    pub key: String,
    pub degree: usize,
    pub in_degree: usize,
    pub out_degree: usize,
    pub centrality: f32,
    pub community: usize,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct DependencyCycle {
    pub nodes: Vec<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Default)]
pub struct GraphAnalytics {
    pub god_nodes: Vec<GodNode>,
    pub cycles: Vec<DependencyCycle>,
    pub cross_community_edges: usize,
    pub isolated_count: usize,
}

/// Computes architectural analytics over a resolved document graph.
/// `node_keys` must match the path keys used in `edges`; `communities` comes
/// from [`compute_communities`] so both analyses agree on clustering.
pub fn analyze_graph(
    node_keys: &[String],
    edges: &[ResolvedEdge],
    communities: &HashMap<String, usize>,
) -> GraphAnalytics {
    let n = node_keys.len();
    let mut analytics = GraphAnalytics::default();
    if n == 0 {
        return analytics;
    }

    let index_of: HashMap<&str, usize> = node_keys
        .iter()
        .enumerate()
        .map(|(i, key)| (key.as_str(), i))
        .collect();

    let mut adj_out: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut adj_in: Vec<Vec<usize>> = vec![Vec::new(); n];
    for edge in edges {
        if let (Some(&a), Some(&b)) = (
            index_of.get(edge.source.as_str()),
            index_of.get(edge.target.as_str()),
        ) {
            adj_out[a].push(b);
            adj_in[b].push(a);
        }
    }

    let total_edges: usize = adj_out.iter().map(|list| list.len()).sum();
    let avg_degree = (total_edges * 2) as f32 / n as f32;

    // --- god nodes: high-degree architectural hubs ---
    for (i, key) in node_keys.iter().enumerate() {
        let out_deg = adj_out[i].len();
        let in_deg = adj_in[i].len();
        let degree = out_deg + in_deg;
        let is_god = degree >= 3 && (degree as f32) >= (avg_degree * 1.3).max(3.0);
        if !is_god {
            continue;
        }
        let role = if in_deg >= out_deg * 2 {
            "Foundation / Core Dependency"
        } else if out_deg >= in_deg * 2 {
            "Orchestrator / Dispatcher"
        } else {
            "Architectural Hub"
        };
        analytics.god_nodes.push(GodNode {
            key: key.clone(),
            degree,
            in_degree: in_deg,
            out_degree: out_deg,
            centrality: if n > 1 {
                degree as f32 / (2.0 * (n - 1) as f32)
            } else {
                0.0
            },
            community: communities.get(key).copied().unwrap_or(0),
            role: role.into(),
        });
    }
    analytics
        .god_nodes
        .sort_by(|a, b| b.degree.cmp(&a.degree).then_with(|| a.key.cmp(&b.key)));
    analytics.god_nodes.truncate(12);

    // --- dependency cycles: directed 2- and 3-node loops ---
    let mut seen_cycles: HashSet<String> = HashSet::new();
    for u in 0..n {
        for &v in &adj_out[u] {
            if v == u {
                continue;
            }
            if adj_out[v].contains(&u) {
                let mut pair = [u, v];
                pair.sort();
                let key = format!("{}<->{}", pair[0], pair[1]);
                if seen_cycles.insert(key) {
                    analytics.cycles.push(DependencyCycle {
                        nodes: vec![node_keys[u].clone(), node_keys[v].clone()],
                        length: 2,
                    });
                }
            } else {
                for &w in &adj_out[v] {
                    if w != u && w != v && adj_out[w].contains(&u) {
                        let mut triad = [u, v, w];
                        triad.sort();
                        let key = format!("{}<->{}<->{}", triad[0], triad[1], triad[2]);
                        if seen_cycles.insert(key) {
                            analytics.cycles.push(DependencyCycle {
                                nodes: vec![
                                    node_keys[u].clone(),
                                    node_keys[v].clone(),
                                    node_keys[w].clone(),
                                ],
                                length: 3,
                            });
                        }
                    }
                }
            }
        }
    }

    // --- cross-community coupling + isolation ---
    for edge in edges {
        let source_community = communities.get(&edge.source).copied().unwrap_or(0);
        let target_community = communities.get(&edge.target).copied().unwrap_or(0);
        if source_community != target_community {
            analytics.cross_community_edges += 1;
        }
    }
    analytics.isolated_count = (0..n)
        .filter(|&i| adj_out[i].is_empty() && adj_in[i].is_empty())
        .count();

    analytics
}
