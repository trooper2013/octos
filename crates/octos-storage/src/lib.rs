use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
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
    /// Enforces strict defensive bounds: if query_vector does not contain exactly 384 elements,
    /// logs an error warning and returns an empty result set.
    pub fn search(&self, query_vector: &[f32], limit: usize) -> Vec<&KnowledgeNode> {
        if query_vector.len() != 384 {
            eprintln!(
                "[SYSTEM LOG] [STORAGE] [ERROR] Bounds check failed: query_vector has {} dimensions (expected 384). Aborting search safely.",
                query_vector.len()
            );
            return Vec::new();
        }

        if self.nodes.is_empty() {
            return Vec::new();
        }

        let mut scored_nodes: Vec<(f32, &KnowledgeNode)> = self
            .nodes
            .iter()
            .filter(|node| {
                if node.vector.len() != 384 {
                    eprintln!(
                        "[SYSTEM LOG] [STORAGE] [WARNING] Skipping node ID: {} - invalid vector dimension of {} (expected 384).",
                        node.id,
                        node.vector.len()
                    );
                    false
                } else {
                    true
                }
            })
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

    /// Flushes the entire VectorStore table directly to the disk as JSON.
    pub fn save_to_disk(&self, path: &str) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Populates the VectorStore by reading and deserializing the DB from disk.
    /// If the database file does not exist, initializes an empty VectorStore cleanly.
    pub fn load_from_disk(path: &str) -> Result<Self, std::io::Error> {
        let path_ref = Path::new(path);
        if !path_ref.exists() {
            return Ok(Self::new());
        }
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let store = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(store)
    }
}

/// Computes the cosine similarity of two 384-dimensional slices.
/// Returns 0.0 if either slice is not exactly 384 dimensions.
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != 384 || v2.len() != 384 {
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
