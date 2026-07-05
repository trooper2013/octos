use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use octos_storage::{KnowledgeNode, VectorStore};

/// Background task that automatically intercepts input strings, slices them,
/// generates mock embeddings, and stores them in the persistent VectorStore.
pub async fn start_ingestion_daemon(
    mut rx: mpsc::Receiver<String>,
    vector_store: Arc<RwLock<VectorStore>>,
) {
    println!("[SYSTEM LOG] [INGESTION DAEMON] Ingestion task started.");
    while let Some(input_text) = rx.recv().await {
        println!(
            "[SYSTEM LOG] [INGESTION DAEMON] Intercepted user context: \"{}\"",
            input_text
        );

        // Slice text into chunks (e.g. split by sentences or split by commas/semicolons)
        let chunks: Vec<&str> = input_text
            .split(|c| c == '.' || c == ';' || c == ',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for chunk in chunks {
            // Generate mock 4D embedding
            let vector = generate_mock_embedding(chunk);
            let node_id = Uuid::new_v4();

            println!(
                "[SYSTEM LOG] [INGESTION DAEMON] Ingesting chunk: \"{}\" | Vector: {:?}",
                chunk, vector
            );

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string());

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "terminal_history".to_string());
            metadata.insert("timestamp_epoch".to_string(), timestamp);

            let node = KnowledgeNode {
                id: node_id,
                vector,
                content: chunk.to_string(),
                metadata,
            };

            // Write dynamically to VectorStore
            let mut store = vector_store.write().await;
            store.insert(node);
        }
    }
    println!("[SYSTEM LOG] [INGESTION DAEMON] Ingestion task terminated.");
}

/// Generates a reproducible mock 4D embedding vector normalized to unit length.
pub fn generate_mock_embedding(text: &str) -> Vec<f32> {
    let chars = text.chars().count() as f32;
    if chars == 0.0 {
        return vec![0.0, 0.0, 0.0, 0.0];
    }
    let vowels = text.chars().filter(|c| "aeiouAEIOU".contains(*c)).count() as f32;
    let consonants = text.chars().filter(|c| c.is_alphabetic() && !"aeiouAEIOU".contains(*c)).count() as f32;
    let special = chars - vowels - consonants;
    
    // Create base 4D vector
    let mut vec = vec![vowels / chars, consonants / chars, special / chars, 0.5];
    
    // Normalize to unit length for accurate cosine similarity
    let norm = (vec[0]*vec[0] + vec[1]*vec[1] + vec[2]*vec[2] + vec[3]*vec[3]).sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}
