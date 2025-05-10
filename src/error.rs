use thiserror::Error;

/// Custom error types for the API Gateway
#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),
    
    #[error("Routing error: {0}")]
    Routing(String),
    
    #[error("Proxy configuration error: {0}")]
    ProxyConfig(String),
    
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),
    
    #[error("TLS configuration error: {0}")]
    TlsConfig(String),
}
