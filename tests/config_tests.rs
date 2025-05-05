use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use rust_api_gateway::{ServerConfig, GatewayError};

#[test]
fn test_load_config_from_env() {
    // Create a temporary YAML file for the test
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join("test_proxies.yaml");
    
    let yaml_content = "proxies: []";
    let mut file = File::create(&yaml_path).unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    
    // Set up test environment variables
    env::set_var("HTTP_PORT", "9090");
    env::set_var("HTTPS_PORT", "9443");
    env::set_var("TLS_CERT_PATH", "/path/to/cert.pem");
    env::set_var("TLS_KEY_PATH", "/path/to/key.pem");
    env::set_var("PROXY_CONFIG_PATH", yaml_path.to_str().unwrap());
    
    // Load config
    let config = ServerConfig::from_env().unwrap();
    
    // Verify config
    assert_eq!(config.http_port, 9090);
    assert_eq!(config.https_port, 9443);
    assert_eq!(config.tls_cert_path, Some(PathBuf::from("/path/to/cert.pem")));
    assert_eq!(config.tls_key_path, Some(PathBuf::from("/path/to/key.pem")));
    assert_eq!(config.proxy_config_path, yaml_path);
    
    // Clean up
    env::remove_var("HTTP_PORT");
    env::remove_var("HTTPS_PORT");
    env::remove_var("TLS_CERT_PATH");
    env::remove_var("TLS_KEY_PATH");
    env::remove_var("PROXY_CONFIG_PATH");
    std::fs::remove_file(yaml_path).ok();
}

#[test]
fn test_load_config_defaults() {
    // Create a temporary YAML file for the test
    let temp_dir = std::env::temp_dir();
    let yaml_path = temp_dir.join("test_proxies_defaults.yaml");
    
    let yaml_content = "proxies: []";
    let mut file = File::create(&yaml_path).unwrap();
    file.write_all(yaml_content.as_bytes()).unwrap();
    
    // Remove all non-required environment variables
    env::remove_var("HTTP_PORT");
    env::remove_var("HTTPS_PORT");
    env::remove_var("TLS_CERT_PATH");
    env::remove_var("TLS_KEY_PATH");
    
    // Set only the required PROXY_CONFIG_PATH to our temporary file
    env::set_var("PROXY_CONFIG_PATH", yaml_path.to_str().unwrap());
    
    // Load config
    let config = ServerConfig::from_env().unwrap();
    
    // Verify defaults - default_http_port() is 8080 according to our implementation
    assert_eq!(config.http_port, 8080);
    assert_eq!(config.https_port, ServerConfig::default_https_port());
    assert_eq!(config.tls_cert_path, None);
    assert_eq!(config.tls_key_path, None);
    assert_eq!(config.proxy_config_path, yaml_path);
    
    // Clean up
    env::remove_var("PROXY_CONFIG_PATH");
    std::fs::remove_file(yaml_path).ok();
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
