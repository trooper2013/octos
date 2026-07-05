use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A node in the non-hierarchical vector filesystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: Uuid,
    pub vector: Vec<f32>,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// An in-memory vector database simulating the vector file system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorStore {
    pub nodes: Vec<KnowledgeNode>,
}

impl VectorStore {
    /// Creates a new empty VectorStore.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Inserts a knowledge node into the database.
    pub fn insert(&mut self, node: KnowledgeNode) {
        self.nodes.push(node);
    }

    /// Searches the vector database by calculating cosine similarity against the query vector.
    /// Returns reference to nodes sorted in descending order of similarity.
    pub fn search(&self, query_vector: &[f32], limit: usize) -> Vec<&KnowledgeNode> {
        if query_vector.is_empty() || self.nodes.is_empty() {
            return Vec::new();
        }

        let mut scored_nodes: Vec<(f32, &KnowledgeNode)> = self
            .nodes
            .iter()
            .map(|node| {
                let score = cosine_similarity(&node.vector, query_vector);
                (score, node)
            })
            .collect();

        // Sort descending by score. We use partial_cmp to handle floats, falling back to Equal.
        scored_nodes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored_nodes
            .into_iter()
            .take(limit)
            .map(|(_, node)| node)
            .collect()
    }
}

/// Computes the cosine similarity of two f32 slices.
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
    let norm_v1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
    let norm_v2: f32 = v2.iter().map(|a| a * a).sum::<f32>().sqrt();

    if norm_v1 == 0.0 || norm_v2 == 0.0 {
        return 0.0;
    }

    dot_product / (norm_v1 * norm_v2)
}

pub mod persistent_graph;
