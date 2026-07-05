use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::VectorStore;

/// Serializes the vector database into a binary format and saves it asynchronously to disk.
pub async fn save_to_disk(
    store: &VectorStore,
    path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes = bincode::serialize(store)?;
    let mut file = File::create(path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;
    Ok(())
}

/// Asynchronously loads and deserializes the vector database from disk.
/// If the file does not exist, it initializes a fresh VectorStore database node tree.
pub async fn load_from_disk(
    path: &str,
) -> Result<VectorStore, Box<dyn std::error::Error + Send + Sync>> {
    let path_ref = Path::new(path);
    if !path_ref.exists() {
        return Ok(VectorStore::new());
    }
    let mut file = File::open(path).await?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;
    let store = bincode::deserialize(&bytes)?;
    Ok(store)
}
