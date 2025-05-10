use std::path::PathBuf;
use tempfile;
use rust_api_gateway::load_proxy_config;

struct TestEnv {
    _temp_dir: tempfile::TempDir,
    config_path: PathBuf,
}

fn setup_test() -> TestEnv {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create test config file
    let config_path = temp_dir.path().join("proxies.yaml");
    std::fs::write(&config_path, r#"
- id: "test-proxy"
  listen_path: "/api"
  backend_protocol: "http"
  backend_host: "localhost"
  backend_port: 8080
  backend_path: "/"
  strip_listen_path: false
  preserve_host_header: false
  backend_connect_timeout_ms: 3000
  backend_read_timeout_ms: 30000
  backend_write_timeout_ms: 30000
"#).unwrap();
    
    TestEnv {
        _temp_dir: temp_dir,
        config_path,
    }
}

#[test]
fn test_load_proxy_config_valid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("proxies.yaml");
    std::fs::write(&config_path, r#"
- id: "test-proxy"
  listen_path: "/api"
  backend_protocol: "http"
  backend_host: "localhost"
  backend_port: 8080
  backend_path: "/"
  strip_listen_path: false
  preserve_host_header: false
  backend_connect_timeout_ms: 3000
  backend_read_timeout_ms: 30000
  backend_write_timeout_ms: 30000
"#).unwrap();
    
    let result = load_proxy_config(&config_path);
    assert!(result.is_ok(), "Failed to load valid config: {:?}", result.err());
    
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 1);
    assert_eq!(proxies[0].id, "test-proxy");
}

#[test]
fn test_load_proxy_config_invalid_path() {
    let yaml_path = PathBuf::from("/path/that/definitely/doesnt/exist/proxies.yaml");
    let result = load_proxy_config(&yaml_path);
    assert!(result.is_err());
}

#[test]
fn test_load_empty_proxy_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("proxies.yaml");
    std::fs::write(&config_path, "[]").unwrap();
    
    let result = load_proxy_config(&config_path);
    assert!(result.is_ok(), "Failed to load empty config: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_load_valid_proxy_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let yaml_path = temp_dir.path().join("proxies.yaml");
    std::fs::write(&yaml_path, r#"
- id: "test-proxy"
  listen_path: "/api"
  backend_protocol: "http"
  backend_host: "localhost"
  backend_port: 8080
  backend_path: "/"
  strip_listen_path: false
  preserve_host_header: false
  backend_connect_timeout_ms: 3000
  backend_read_timeout_ms: 30000
  backend_write_timeout_ms: 30000
"#).unwrap();
    
    let result = load_proxy_config(&yaml_path);
    assert!(result.is_ok(), "Failed to load valid config: {:?}", result.err());
    let proxies = result.unwrap();
    assert_eq!(proxies.len(), 1);
    assert_eq!(proxies[0].id, "test-proxy");
}