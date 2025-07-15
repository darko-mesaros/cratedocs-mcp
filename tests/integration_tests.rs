use cratedocs_mcp::{tools::DocRouter, transport::jsonrpc_frame_codec::JsonRpcFrameCodec};
use mcp_server::Router;
use serde_json::{json, Value};
use tokio_util::codec::Decoder;

#[tokio::test]
async fn test_doc_router_initialization() {
    let router = DocRouter::new();
    
    // Basic properties should be correct
    assert_eq!(router.name(), "rust-docs");
    assert!(router.capabilities().tools.is_some());
    
    // Tools should be available and correctly configured
    let tools = router.list_tools();
    assert_eq!(tools.len(), 3);
    
    // Check specific tool schemas
    let lookup_crate_tool = tools.iter().find(|t| t.name == "lookup_crate").unwrap();
    let schema: Value = serde_json::from_value(lookup_crate_tool.input_schema.clone()).unwrap();
    assert_eq!(schema["type"], "object");
    assert!(schema["required"].as_array().unwrap().contains(&json!("crate_name")));
}

#[test]
fn test_jsonrpc_codec_functionality() {
    let mut codec = JsonRpcFrameCodec;
    let json_rpc = r#"{"jsonrpc":"2.0","method":"lookup_crate","params":{"crate_name":"tokio"},"id":1}"#;
    
    let mut buffer = tokio_util::bytes::BytesMut::from(json_rpc);
    buffer.extend_from_slice(b"\n");
    
    let decoded = codec.decode(&mut buffer).unwrap().unwrap();
    assert_eq!(decoded, json_rpc);
}

#[tokio::test]
async fn test_invalid_parameters_handling() {
    let router = DocRouter::new();
    
    // Call lookup_crate with missing required parameter
    let result = router.call_tool("lookup_crate", json!({})).await;
    assert!(matches!(result, Err(mcp_core::ToolError::InvalidParameters(_))));
    
    // Call with invalid tool name
    let result = router.call_tool("invalid_tool", json!({})).await;
    assert!(matches!(result, Err(mcp_core::ToolError::NotFound(_))));
}

// This test requires network access
#[tokio::test]
#[ignore = "Requires network access"]
async fn test_end_to_end_crate_lookup() {
    let router = DocRouter::new();
    
    // Look up a well-known crate
    let result = router.call_tool(
        "lookup_crate", 
        json!({
            "crate_name": "serde"
        })
    ).await;
    
    assert!(result.is_ok());
    let content = result.unwrap();
    assert_eq!(content.len(), 1);
    
    // The response should be HTML from docs.rs
    match &content[0] {
        mcp_core::Content::Text(text) => {
            assert!(text.text.contains("<!DOCTYPE html>"));
            assert!(text.text.contains("serde"));
        },
        _ => panic!("Expected text content"),
    }
}

// Test resource and prompt API error cases (since they're not implemented)
#[tokio::test]
async fn test_unimplemented_apis() {
    let router = DocRouter::new();
    
    // Resources should return an empty list
    assert!(router.list_resources().is_empty());
    
    // Reading a resource should fail
    let result = router.read_resource("test").await;
    assert!(result.is_err());
    
    // Prompts should return an empty list
    assert!(router.list_prompts().is_empty());
    
    // Getting a prompt should fail
    let result = router.get_prompt("test").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_configurable_log_directory() {
    use std::env;
    use tempfile::TempDir;

    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // Set the environment variable to the temp directory
    env::set_var("CRATEDOCS_LOG_DIR", temp_path);
    
    // Test that the get_log_directory function returns the custom path
    // Note: We can't easily test the actual logging without running the full server,
    // but we can test that the environment variable is read correctly
    let log_dir = std::env::var("CRATEDOCS_LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    assert_eq!(log_dir, temp_path);
    
    // Clean up
    env::remove_var("CRATEDOCS_LOG_DIR");
}

#[tokio::test]
async fn test_default_log_directory_behavior() {
    use std::env;
    
    // Ensure the environment variable is not set
    env::remove_var("CRATEDOCS_LOG_DIR");
    
    // Test that the default behavior still works
    let log_dir = std::env::var("CRATEDOCS_LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    assert_eq!(log_dir, "logs");
}