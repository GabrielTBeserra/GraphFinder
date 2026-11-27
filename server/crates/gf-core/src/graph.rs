//! In-memory graph primitives for notes, files and symbols.

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    adjacency: HashMap<String, Vec<String>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, source: &str, target: &str, label: Option<String>) {
        self.adjacency
            .entry(source.to_string())
            .or_default()
            .push(target.to_string());
        self.adjacency
            .entry(target.to_string())
            .or_default()
            .push(source.to_string());
        self.edges.push(GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            label,
        });
    }

    pub fn neighbors(&self, node: &str) -> Vec<&GraphNode> {
        self.adjacency
            .get(node)
            .into_iter()
            .flatten()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Breadth-first shortest path. Edges are treated as undirected for
    /// architectural navigation, matching how agents explore dependencies.
    pub fn find_path(&self, source: &str, target: &str) -> Option<Vec<String>> {
        if source == target {
            return self
                .nodes
                .contains_key(source)
                .then(|| vec![source.to_string()]);
        }
        let mut visited = HashSet::from([source.to_string()]);
        let mut parent = HashMap::new();
        let mut queue = VecDeque::from([source.to_string()]);

        while let Some(current) = queue.pop_front() {
            for next in self.adjacency.get(&current).into_iter().flatten() {
                if !visited.insert(next.clone()) {
                    continue;
                }
                parent.insert(next.clone(), current.clone());
                if next == target {
                    let mut path = vec![target.to_string()];
                    let mut cursor = target;
                    while let Some(previous) = parent.get(cursor) {
                        path.push(previous.clone());
                        cursor = previous;
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(next.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> GraphNode {
        GraphNode {
            id: id.into(),
            title: id.into(),
            kind: "test".into(),
            path: None,
        }
    }

    #[test]
    fn builds_neighborhoods() {
        let mut graph = KnowledgeGraph::new();
        graph.add_node(node("a"));
        graph.add_node(node("b"));
        graph.add_edge("a", "b", None);
        assert_eq!(graph.neighbors("a")[0].id, "b");
        assert_eq!(graph.neighbors("b")[0].id, "a");
    }

    #[test]
    fn finds_shortest_path() {
        let mut graph = KnowledgeGraph::new();
        for id in ["api", "service", "db", "docs"] {
            graph.add_node(node(id));
        }
        graph.add_edge("api", "service", None);
        graph.add_edge("service", "db", None);
        graph.add_edge("api", "db", None);
        assert_eq!(
            graph.find_path("api", "db"),
            Some(vec!["api".into(), "db".into()])
        );
        assert_eq!(graph.find_path("docs", "db"), None);
    }
}
