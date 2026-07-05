use octos_init::{mount_vfs, model_loader};

#[test]
fn test_vfs_mounts_execute() {
    let result = mount_vfs();
    assert!(result.is_ok());
}

#[test]
fn test_model_loader_allocations() {
    let size_bytes = 100 * 1024 * 1024; // 100 MB
    let result = model_loader::stage_model_weights("test-model.onnx", size_bytes);
    
    assert!(result.is_ok());
    let weights = result.unwrap();
    assert_eq!(weights.name, "test-model.onnx");
    assert_eq!(weights.size_bytes, size_bytes);
    
    let range = weights.allocated_address_range;
    assert_eq!(range.1 - range.0, size_bytes as usize);
}
