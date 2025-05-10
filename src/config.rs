use std::env;
use std::path::PathBuf;

use serde::Deserialize;
use tracing::{error, info};

use crate::error::GatewayError;

/// Server configuration struct for the API Gateway
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub http_port: u16,
    pub https_port: u16,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
    pub proxy_config_path: PathBuf,
}

impl ServerConfig {
    /// Default HTTP port helper function
    pub fn default_http_port() -> u16 {
        8080
    }
    
    /// Default HTTPS port helper function
    pub fn default_https_port() -> u16 {
        8443
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, GatewayError> {
        // Load HTTP port with default
        let http_port = match env::var("HTTP_PORT") {
            Ok(port_str) if !port_str.is_empty() => match port_str.parse::<u16>() {
                Ok(port) => port,
                Err(_) => {
                    error!("Invalid HTTP_PORT value: {}, using default", port_str);
                    Self::default_http_port()
                }
            },
            Ok(_) | Err(env::VarError::NotPresent) => Self::default_http_port(),
            Err(e) => {
                error!("Error reading HTTP_PORT: {}, using default", e);
                Self::default_http_port()
            }
        };
        
        // Load HTTPS port with default
        let https_port = match env::var("HTTPS_PORT") {
            Ok(port_str) if !port_str.is_empty() => match port_str.parse::<u16>() {
                Ok(port) => port,
                Err(_) => {
                    error!("Invalid HTTPS_PORT value: {}, using default", port_str);
                    Self::default_https_port()
                }
            },
            Ok(_) | Err(env::VarError::NotPresent) => Self::default_https_port(),
            Err(e) => {
                error!("Error reading HTTPS_PORT: {}, using default", e);
                Self::default_https_port()
            }
        };
        
        // Load TLS cert path
        let tls_cert_path = match env::var("TLS_CERT_PATH") {
            Ok(path) => Some(PathBuf::from(&path)),
            Err(_) => {
                info!("TLS_CERT_PATH not set, TLS will be disabled");
                None
            }
        };
        
        // Load TLS key path
        let tls_key_path = match env::var("TLS_KEY_PATH") {
            Ok(path) => Some(PathBuf::from(&path)),
            Err(_) => {
                info!("TLS_KEY_PATH not set, TLS will be disabled");
                None
            }
        };
        
        // Load proxy configuration path (required)
        let proxy_config_path = match env::var("PROXY_CONFIG_PATH") {
            Ok(path) => {
                let path_buf = PathBuf::from(&path);
                
                // Check if the file actually exists
                if !path_buf.exists() {
                    let error_msg = format!("PROXY_CONFIG_PATH points to a non-existent file: {}", path);
                    error!("{}", error_msg);
                    return Err(GatewayError::Config(error_msg));
                }
                
                path_buf
            },
            Err(_) => {
                let error_msg = "PROXY_CONFIG_PATH environment variable is required but not set";
                error!("{}", error_msg);
                return Err(GatewayError::Config(error_msg.to_string()));
            }
        };
        
        // Create and return the config
        Ok(ServerConfig {
            http_port,
            https_port,
            tls_cert_path: if tls_cert_path.is_some() && tls_cert_path.as_ref().unwrap().to_str().unwrap().is_empty() {
                None
            } else {
                tls_cert_path
            },
            tls_key_path: if tls_key_path.is_some() && tls_key_path.as_ref().unwrap().to_str().unwrap().is_empty() {
                None
            } else {
                tls_key_path
            },
            proxy_config_path,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u16,
    pub proxy_config_path: PathBuf,
}

pub fn load_config() -> Result<Config, GatewayError> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .map_err(|_| GatewayError::Config("Invalid PORT value".to_string()))?;
    
    let proxy_config_path = env::var("PROXY_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("config/proxies.yaml"));
    
    Ok(Config {
        port,
        proxy_config_path,
    })
}
