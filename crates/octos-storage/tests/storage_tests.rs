use std::collections::HashMap;
use uuid::Uuid;
use octos_storage::{cosine_similarity, KnowledgeNode, VectorStore};

/// Helper to generate a normalized 384-dimensional vector with deterministic values.
fn make_384_vector(v0: f32, v1: f32) -> Vec<f32> {
    let mut vec = vec![0.0f32; 384];
    vec[0] = v0;
    vec[1] = v1;
    let norm = (v0 * v0 + v1 * v1).sqrt();
    if norm > 0.0 {
        vec[0] /= norm;
        vec[1] /= norm;
    }
    vec
}

#[test]
fn test_cosine_similarity_exact_match() {
    let v1 = make_384_vector(1.0, 2.0);
    let v2 = make_384_vector(1.0, 2.0);
    let similarity = cosine_similarity(&v1, &v2);
    assert!((similarity - 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    let v1 = make_384_vector(1.0, 0.0);
    let v2 = make_384_vector(0.0, 1.0);
    let similarity = cosine_similarity(&v1, &v2);
    assert!((similarity - 0.0).abs() < 1e-6);
}

#[test]
fn test_vector_search_ranking() {
    let mut store = VectorStore::new();
    
    // Ingest 5 nodes with different 384D directions
    for i in 1..=5 {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), i.to_string());
        
        let vector = match i {
            1 => make_384_vector(1.0, 0.0), // Perpendicular to query
            2 => make_384_vector(0.0, 1.0), // Exact match to query [0, 1]
            3 => make_384_vector(0.0, 0.5), // Matches query direction perfectly
            4 => make_384_vector(0.5, 0.5), // 45 degrees similarity
            5 => make_384_vector(-1.0, 0.0), // Opposite/Orthogonal mapping
            _ => unreachable!(),
        };

        store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector,
            content: format!("Node {}", i),
            metadata,
        });
    }

    let query = make_384_vector(0.0, 1.0);
    let results = store.search(&query, 5);

    assert_eq!(results.len(), 5);
    // Node 2 and Node 3 both point exactly in the query direction, similarity 1.0
    assert!(results[0].content == "Node 2" || results[0].content == "Node 3");
    assert!(results[1].content == "Node 2" || results[1].content == "Node 3");
    // Third best should be Node 4 (similarity ~0.707)
    assert_eq!(results[2].content, "Node 4");
}

#[test]
fn test_search_limit_enforcement() {
    let mut store = VectorStore::new();
    for i in 1..=5 {
        store.insert(KnowledgeNode {
            id: Uuid::new_v4(),
            vector: make_384_vector(1.0, 0.0),
            content: format!("Node {}", i),
            metadata: HashMap::new(),
        });
    }

    let results = store.search(&make_384_vector(1.0, 0.0), 2);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_persistence_roundtrip() {
    let test_path = "C:\\octos\\octos\\temp_test_store.db";
    
    let mut store = VectorStore::new();
    let node_id = Uuid::new_v4();
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());
    
    store.insert(KnowledgeNode {
        id: node_id,
        vector: make_384_vector(0.1, 0.2),
        content: "Test JSON persistence".to_string(),
        metadata,
    });

    store.save_to_disk(test_path).expect("Failed to save store");

    let loaded = VectorStore::load_from_disk(test_path).expect("Failed to load store");
    assert_eq!(loaded.nodes.len(), 1);
    assert_eq!(loaded.nodes[0].id, node_id);
    assert_eq!(loaded.nodes[0].content, "Test JSON persistence");
    assert_eq!(loaded.nodes[0].vector, make_384_vector(0.1, 0.2));
    assert_eq!(loaded.nodes[0].metadata.get("source").unwrap(), "test");

    let _ = std::fs::remove_file(test_path);
}

#[test]
fn test_search_bounds_checking() {
    let mut store = VectorStore::new();
    store.insert(KnowledgeNode {
        id: Uuid::new_v4(),
        vector: make_384_vector(1.0, 0.0),
        content: "Valid 384D node".to_string(),
        metadata: HashMap::new(),
    });

    // Query with an invalid 4-dimensional vector
    let invalid_query = vec![1.0, 0.0, 0.0, 0.0];
    let results = store.search(&invalid_query, 5);
    
    // Bounds check must log warning and return empty vector without crashing
    assert!(results.is_empty());
}
