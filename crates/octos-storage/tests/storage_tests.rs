use std::collections::HashMap;
use uuid::Uuid;
use octos_storage::{cosine_similarity, KnowledgeNode, VectorStore};

#[test]
fn test_cosine_similarity_exact_match() {
    let v1 = vec![1.0, 2.0, 3.0, 4.0];
    let v2 = vec![1.0, 2.0, 3.0, 4.0];
    let similarity = cosine_similarity(&v1, &v2);
    assert!((similarity - 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    let similarity = cosine_similarity(&v1, &v2);
    assert!((similarity - 0.0).abs() < 1e-6);
}

#[test]
fn test_vector_search_ranking() {
    let mut store = VectorStore::new();
    
    // Ingress 5 nodes with different directions
    for i in 1..=5 {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), i.to_string());
        
        let vector = match i {
            1 => vec![1.0, 0.0, 0.0, 0.0], // Orthogonal to query [0, 1, 0, 0]
            2 => vec![0.0, 1.0, 0.0, 0.0], // Exact match to query [0, 1, 0, 0]
            3 => vec![0.0, 0.5, 0.5, 0.0], // High similarity
            4 => vec![0.0, 0.0, 1.0, 0.0], // Orthogonal to query
            5 => vec![0.0, 0.1, 0.0, 0.9], // Very low similarity
            _ => unreachable!(),
        };

        store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector,
            content: format!("Node {}", i),
            metadata,
        });
    }

    let query = vec![0.0, 1.0, 0.0, 0.0];
    let results = store.search(&query, 5);

    assert_eq!(results.len(), 5);
    // Best match should be Node 2 (exact match, similarity 1.0)
    assert_eq!(results[0].content, "Node 2");
    // Second best should be Node 3 (similarity ~0.707)
    assert_eq!(results[1].content, "Node 3");
    // Third best should be Node 5 (similarity ~0.11)
    assert_eq!(results[2].content, "Node 5");
    
    // Ensure descending order of score
    let score1 = cosine_similarity(&results[0].vector, &query);
    let score2 = cosine_similarity(&results[1].vector, &query);
    let score3 = cosine_similarity(&results[2].vector, &query);
    let score4 = cosine_similarity(&results[3].vector, &query);
    let score5 = cosine_similarity(&results[4].vector, &query);

    assert!(score1 >= score2);
    assert!(score2 >= score3);
    assert!(score3 >= score4);
    assert!(score4 >= score5);
}

#[test]
fn test_search_limit_enforcement() {
    let mut store = VectorStore::new();
    for i in 1..=5 {
        store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector: vec![1.0, 0.0, 0.0, 0.0],
            content: format!("Node {}", i),
            metadata: HashMap::new(),
        });
    }

    let results = store.search(&[1.0, 0.0, 0.0, 0.0], 2);
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_persistence_roundtrip() {
    use octos_storage::persistent_graph::{save_to_disk, load_from_disk};
    let test_path = "C:\\octos\\octos\\temp_test_store.bin";
    
    let mut store = VectorStore::new();
    let node_id = Uuid::new_v4();
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());
    
    store.insert(KnowledgeNode {
        id: node_id,
        vector: vec![0.1, 0.2, 0.3, 0.4],
        content: "Test persistence content".to_string(),
        metadata,
    });

    save_to_disk(&store, test_path).await.expect("Failed to save store");

    let loaded = load_from_disk(test_path).await.expect("Failed to load store");
    assert_eq!(loaded.nodes.len(), 1);
    assert_eq!(loaded.nodes[0].id, node_id);
    assert_eq!(loaded.nodes[0].content, "Test persistence content");
    assert_eq!(loaded.nodes[0].vector, vec![0.1, 0.2, 0.3, 0.4]);
    assert_eq!(loaded.nodes[0].metadata.get("source").unwrap(), "test");

    let _ = tokio::fs::remove_file(test_path).await;
}

