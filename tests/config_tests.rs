use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use rust_api_gateway::{ServerConfig, GatewayError};
use tempfile;

struct TestEnv {
    _temp_dir: tempfile::TempDir,
    yaml_path: PathBuf,
}

fn setup_test() -> TestEnv {
    // Create unique temp directory and file
    let temp_dir = tempfile::tempdir().unwrap();
    let yaml_path = temp_dir.path().join("proxies.yaml");
    
    let yaml_content = "proxies: []";
    let mut file = File::create(&yaml_path).unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    
    // Clear ALL environment variables
    env::remove_var("HTTP_PORT");
    env::remove_var("HTTPS_PORT");
    env::remove_var("TLS_CERT_PATH");
    env::remove_var("TLS_KEY_PATH");
    env::remove_var("PROXY_CONFIG_PATH");
    
    TestEnv {
        _temp_dir: temp_dir,
        yaml_path,
    }
}

fn teardown_test(test_env: TestEnv) {
    // Clear ALL environment variables again
    env::remove_var("HTTP_PORT");
    env::remove_var("HTTPS_PORT");
    env::remove_var("TLS_CERT_PATH");
    env::remove_var("TLS_KEY_PATH");
    env::remove_var("PROXY_CONFIG_PATH");
}

#[test]
fn test_load_config_defaults() {
    let test_env = setup_test();
    
    // Set only required var
    env::set_var("PROXY_CONFIG_PATH", test_env.yaml_path.to_str().unwrap());
    
    let config = ServerConfig::from_env().unwrap();
    
    // Verify defaults
    assert_eq!(config.http_port, 8080, "HTTP port should default to 8080");
    assert_eq!(config.https_port, 8443, "HTTPS port should default to 8443");
    assert_eq!(config.tls_cert_path, None, "TLS cert path should default to None");
    assert_eq!(config.tls_key_path, None, "TLS key path should default to None");
    assert_eq!(config.proxy_config_path, test_env.yaml_path, "Proxy config path should match");
    
    teardown_test(test_env);
}

#[test]
fn test_load_config_from_env() {
    let test_env = setup_test();
    
    // Set test vars
    env::set_var("HTTP_PORT", "9090");
    env::set_var("HTTPS_PORT", "9443");
    env::set_var("PROXY_CONFIG_PATH", test_env.yaml_path.to_str().unwrap());
    
    let config = ServerConfig::from_env().unwrap();
    
    // Verify config
    assert_eq!(config.http_port, 9090, "HTTP port should match env var");
    assert_eq!(config.https_port, 9443, "HTTPS port should match env var");
    assert_eq!(config.tls_cert_path, None, "TLS cert path should be None");
    assert_eq!(config.tls_key_path, None, "TLS key path should be None");
    assert_eq!(config.proxy_config_path, test_env.yaml_path, "Proxy config path should match");
    
    teardown_test(test_env);
}

#[test]
fn test_load_config_missing_required() {
    // Remove required PROXY_CONFIG_PATH
    env::remove_var("PROXY_CONFIG_PATH");
    
    // Load config should fail
    let result = ServerConfig::from_env();
    assert!(result.is_err());
    
    // Verify error is correct type
    match result {
        Err(GatewayError::Config(msg)) => {
            assert!(msg.contains("PROXY_CONFIG_PATH"));
        },
        _ => panic!("Expected GatewayError::Config"),
    }
}
