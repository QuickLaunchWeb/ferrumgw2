use std::env;
use tempfile;
use rust_api_gateway::config::ServerConfig;

struct TestEnv {
    _temp_dir: tempfile::TempDir,
    config_path: String,
}

fn setup_test() -> TestEnv {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create a dummy config file
    let config_path = temp_dir.path().join("proxies.yaml");
    std::fs::write(&config_path, "proxies: []").unwrap();
    
    // Set required env vars
    env::set_var("PROXY_CONFIG_PATH", config_path.to_str().unwrap());
    env::set_var("HTTP_PORT", "8081");
    env::set_var("HTTPS_PORT", "8444");
    
    TestEnv {
        _temp_dir: temp_dir,
        config_path: config_path.to_str().unwrap().to_string(),
    }
}

fn teardown_test(_test_env: TestEnv) {
    env::remove_var("PROXY_CONFIG_PATH");
    env::remove_var("HTTP_PORT");
    env::remove_var("HTTPS_PORT");
}

#[test]
fn test_load_config_from_env() {
    let test_env = setup_test();
    
    let config = ServerConfig::from_env().unwrap();
    
    assert_eq!(config.http_port, 8081);
    assert_eq!(config.https_port, 8444);
    assert!(config.proxy_config_path.to_str().unwrap().ends_with("proxies.yaml"));
    assert!(config.tls_cert_path.is_none());
    assert!(config.tls_key_path.is_none());
    
    teardown_test(test_env);
}

#[test]
fn test_load_config_defaults() {
    let test_env = setup_test();
    
    let config = ServerConfig::from_env().unwrap();
    
    assert_eq!(config.http_port, 8081);
    assert_eq!(config.https_port, 8444);
    assert!(config.proxy_config_path.to_str().unwrap().ends_with("proxies.yaml"));
    assert!(config.tls_cert_path.is_none());
    assert!(config.tls_key_path.is_none());
    
    teardown_test(test_env);
}

#[test]
fn test_load_config_missing_required() {
    env::remove_var("PROXY_CONFIG_PATH");
    
    let result = ServerConfig::from_env();
    assert!(result.is_err());
    
    // Restore env var for other tests
    setup_test();
}
