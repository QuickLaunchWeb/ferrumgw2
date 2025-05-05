use thiserror::Error;

/// Custom error types for the API Gateway
#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Proxy configuration error: {0}")]
    ProxyConfig(String),
    
    #[error("Router insertion error: {0}")]
    RouterInsert(String),
    
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
    
    #[error("TLS configuration error: {0}")]
    TlsConfig(String),
}
