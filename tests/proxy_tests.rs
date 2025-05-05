use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use rust_api_gateway::{GatewayError, ProxyDefinition, load_proxy_config};
use rust_api_gateway::proxy::build_router;

#[test]
fn test_load_proxy_config_valid() {
    // Create a temporary YAML file with proxy definitions
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join("valid_proxies.yaml");
    
    let yaml_content = r#"
proxies:
  - id: "test-api"
    name: "Test API"
    listen_path: "/api"
    backend_protocol: "http"
    backend_host: "test-host.local"
    backend_port: 8080
    backend_path: "/backend"
    strip_listen_path: true
    preserve_host_header: true
    backend_connect_timeout_ms: 5000
    backend_read_timeout_ms: 60000
    backend_write_timeout_ms: 60000
  
  - id: "minimal-api"
    name: "Minimal API"
    listen_path: "/minimal"
    backend_protocol: "http"
    backend_host: "minimal-host.local"
"#;
    
    let mut file = File::create(&yaml_path).unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    
    // Test loading the proxy configuration
    let result = load_proxy_config(&yaml_path);
    assert!(result.is_ok());
    
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 2);
    
    // Check first proxy with all fields specified
    let full_proxy = &proxies[0];
    assert_eq!(full_proxy.id, "test-api");
    assert_eq!(full_proxy.name, "Test API");
    assert_eq!(full_proxy.listen_path, "/api");
    assert_eq!(full_proxy.backend_protocol, "http");
    assert_eq!(full_proxy.backend_host, "test-host.local");
    assert_eq!(full_proxy.backend_port, 8080);
    assert_eq!(full_proxy.backend_path, "/backend");
    assert_eq!(full_proxy.strip_listen_path, true);
    assert_eq!(full_proxy.preserve_host_header, true);
    assert_eq!(full_proxy.backend_connect_timeout_ms, 5000);
    assert_eq!(full_proxy.backend_read_timeout_ms, 60000);
    assert_eq!(full_proxy.backend_write_timeout_ms, 60000);
    
    // Check second proxy with minimal fields
    let minimal_proxy = &proxies[1];
    assert_eq!(minimal_proxy.id, "minimal-api");
    assert_eq!(minimal_proxy.name, "Minimal API");
    assert_eq!(minimal_proxy.listen_path, "/minimal");
    assert_eq!(minimal_proxy.backend_protocol, "http");
    assert_eq!(minimal_proxy.backend_host, "minimal-host.local");
    
    // Check defaults are applied
    assert_eq!(minimal_proxy.backend_port, 80); // default_backend_port()
    assert_eq!(minimal_proxy.backend_path, "/"); // default_backend_path()
    assert_eq!(minimal_proxy.strip_listen_path, false); // default_strip_listen_path()
    assert_eq!(minimal_proxy.preserve_host_header, false); // default_preserve_host_header()
    assert_eq!(minimal_proxy.backend_connect_timeout_ms, 3000); // default_backend_connect_timeout_ms()
    assert_eq!(minimal_proxy.backend_read_timeout_ms, 30000); // default_backend_read_timeout_ms()
    assert_eq!(minimal_proxy.backend_write_timeout_ms, 30000); // default_backend_write_timeout_ms()
    
    // Clean up
    std::fs::remove_file(yaml_path).ok();
}

#[test]
fn test_load_proxy_config_invalid_yaml() {
    // Create a temporary file with invalid YAML
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join("invalid_proxies.yaml");
    
    let yaml_content = r#"
    This is not valid YAML:
      - missing colon after key
        value: without proper indentation
    "#;
    
    let mut file = File::create(&yaml_path).unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    
    // Test loading the proxy configuration should fail
    let result = load_proxy_config(&yaml_path);
    assert!(result.is_err());
    
    // Verify error is correct type
    match result {
        Err(GatewayError::Yaml(_)) => (), // Success, error is the expected type
        _ => panic!("Expected GatewayError::Yaml"),
    }
    
    // Clean up
    std::fs::remove_file(yaml_path).ok();
}

#[test]
fn test_load_proxy_config_nonexistent_file() {
    // Use a path that definitely doesn't exist
    let yaml_path = PathBuf::from("/path/that/definitely/doesnt/exist/proxies.yaml");
    
    // Test loading the proxy configuration should fail
    let result = load_proxy_config(&yaml_path);
    assert!(result.is_err());
    
    // Verify error is correct type
    match result {
        Err(GatewayError::Io(_)) => (), // Success, error is the expected type
        _ => panic!("Expected GatewayError::Io"),
    }
}

#[test]
fn test_build_router() {
    // Create test proxy definitions
    let proxies = vec![
        ProxyDefinition {
            id: "test-api-1".to_string(),
            name: "Test API 1".to_string(),
            listen_path: "/api/v1".to_string(),
            backend_protocol: "http".to_string(),
            backend_host: "localhost".to_string(),
            backend_port: 8001,
            backend_path: "/internal/api".to_string(),
            strip_listen_path: true,
            preserve_host_header: false,
            backend_connect_timeout_ms: 3000,
            backend_read_timeout_ms: 30000,
            backend_write_timeout_ms: 30000,
        },
        ProxyDefinition {
            id: "test-api-2".to_string(),
            name: "Test API 2".to_string(),
            listen_path: "api/v2".to_string(), // No leading slash, should be added
            backend_protocol: "http".to_string(),
            backend_host: "localhost".to_string(),
            backend_port: 8002,
            backend_path: "/".to_string(),
            strip_listen_path: false,
            preserve_host_header: true,
            backend_connect_timeout_ms: 3000,
            backend_read_timeout_ms: 30000,
            backend_write_timeout_ms: 30000,
        },
        ProxyDefinition {
            id: "wildcard-service".to_string(),
            name: "Wildcard Service".to_string(),
            listen_path: "/service/:id".to_string(),
            backend_protocol: "http".to_string(),
            backend_host: "localhost".to_string(),
            backend_port: 8003,
            backend_path: "/internal/service".to_string(),
            strip_listen_path: false,
            preserve_host_header: false,
            backend_connect_timeout_ms: 3000,
            backend_read_timeout_ms: 30000,
            backend_write_timeout_ms: 30000,
        },
    ];
    
    // Build router
    let result = build_router(&proxies);
    assert!(result.is_ok());
    
    let router = result.unwrap();
    
    // Test exact path matching
    let match_result = router.at("/api/v1");
    assert!(match_result.is_ok());
    let matched = match_result.unwrap();
    assert_eq!(matched.value.id, "test-api-1");
    
    // Test path with added leading slash
    let match_result = router.at("/api/v2");
    assert!(match_result.is_ok());
    let matched = match_result.unwrap();
    assert_eq!(matched.value.id, "test-api-2");
    
    // Test wildcard path matching
    let match_result = router.at("/service/123");
    assert!(match_result.is_ok());
    let matched = match_result.unwrap();
    assert_eq!(matched.value.id, "wildcard-service");
    assert_eq!(matched.params.get("id"), Some("123"));
    
    // Test non-existent path
    let match_result = router.at("/non-existent");
    assert!(match_result.is_err());
}
