use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use octos_storage::{KnowledgeNode, VectorStore};

/// Background task that automatically intercepts input strings, slices them,
/// generates dense 384-dimensional mock embeddings, and stores them in the persistent VectorStore.
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
            // Generate mock dense 384D embedding
            let vector = generate_mock_embedding(chunk);
            let node_id = Uuid::new_v4();

            println!(
                "[SYSTEM LOG] [INGESTION DAEMON] Ingesting chunk: \"{}\" | 384-dim Vector generated.",
                chunk
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

/// Generates a reproducible mock 384-dimensional embedding vector normalized to unit length.
pub fn generate_mock_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; 384];
    if text.is_empty() {
        return vec;
    }

    // Distribute character codes across the 384 elements
    for (i, c) in text.chars().enumerate() {
        let val = c as u32;
        let idx = (val as usize + i) % 384;
        vec[idx] += 1.0;
    }

    // Inject density modifier
    for i in 0..384 {
        let frequency_modifier = (text.len() as f32 * (i as f32).sin()).abs();
        vec[i] += frequency_modifier % 1.5;
    }

    // Normalize to unit length for accurate cosine similarity
    let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
    vec
}
