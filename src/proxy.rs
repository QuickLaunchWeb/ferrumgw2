use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use hyper::{Client, client::HttpConnector};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use matchit::Router;
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::error::GatewayError;

/// Proxy configuration file structure that matches YAML format
#[derive(Debug, Clone, Deserialize)]
pub struct ProxyConfigFile {
    pub proxies: Vec<ProxyDefinition>,
}

/// Proxy definition structure for routing rules
#[derive(Debug, Clone, Deserialize)]
pub struct ProxyDefinition {
    pub id: String,
    pub name: String,
    pub listen_path: String,
    pub backend_protocol: String,
    pub backend_host: String,
    
    #[serde(default = "default_backend_port")]
    pub backend_port: u16,
    
    #[serde(default = "default_backend_path")]
    pub backend_path: String,
    
    #[serde(default = "default_strip_listen_path")]
    pub strip_listen_path: bool,
    
    #[serde(default = "default_preserve_host_header")]
    pub preserve_host_header: bool,
    
    #[serde(default = "default_backend_connect_timeout_ms")]
    pub backend_connect_timeout_ms: u64,
    
    #[serde(default = "default_backend_read_timeout_ms")]
    pub backend_read_timeout_ms: u64,
    
    #[serde(default = "default_backend_write_timeout_ms")]
    pub backend_write_timeout_ms: u64,
}

/// Default values for ProxyDefinition fields
pub fn default_backend_port() -> u16 {
    80
}

pub fn default_backend_path() -> String {
    "/".to_string()
}

pub fn default_strip_listen_path() -> bool {
    false
}

pub fn default_preserve_host_header() -> bool {
    false
}

pub fn default_backend_connect_timeout_ms() -> u64 {
    3000
}

pub fn default_backend_read_timeout_ms() -> u64 {
    30000
}

pub fn default_backend_write_timeout_ms() -> u64 {
    30000
}

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub router: Arc<Router<ProxyDefinition>>,
    pub http_client: Client<HttpConnector>,
    pub https_client: Client<HttpsConnector<HttpConnector>>,
}

impl AppState {
    /// Create a new AppState with a router populated from the provided proxy definitions
    pub fn new(proxies: &[ProxyDefinition]) -> Result<Self, GatewayError> {
        let router = build_router(proxies)?;
        
        // This should now be created in main.rs and passed to this function
        let http_client = Client::new();
        
        // Create the HTTPS connector using rustls
        let https_connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build();
            
        let https_client = Client::builder().build(https_connector);
        
        Ok(Self {
            router: Arc::new(router),
            http_client,
            https_client,
        })
    }
}

/// Load proxy configuration from a YAML file
pub fn load_proxy_config(path: &PathBuf) -> Result<Vec<ProxyDefinition>, GatewayError> {
    let mut file = File::open(path).map_err(|e| {
        let error_msg = format!("Failed to open proxy config file at {}: {}", path.display(), e);
        error!("{}", error_msg);
        GatewayError::Io(e)
    })?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| {
        let error_msg = format!("Failed to read proxy config file at {}: {}", path.display(), e);
        error!("{}", error_msg);
        GatewayError::Io(e)
    })?;
    
    let proxy_config: ProxyConfigFile = serde_yaml::from_str(&contents).map_err(|e| {
        let error_msg = format!("Failed to parse YAML in proxy config file: {}", e);
        error!("{}", error_msg);
        GatewayError::Yaml(e)
    })?;
    
    if proxy_config.proxies.is_empty() {
        let warning_msg = "No proxy definitions found in configuration file";
        info!("{}", warning_msg);
    }
    
    Ok(proxy_config.proxies)
}

/// Build a router from a list of proxy definitions
pub fn build_router(proxies: &[ProxyDefinition]) -> Result<Router<ProxyDefinition>, GatewayError> {
    let mut router = Router::new();
    
    for proxy in proxies {
        // Ensure the path starts with a forward slash
        let path = if !proxy.listen_path.starts_with('/') {
            format!("/{}", proxy.listen_path)
        } else {
            proxy.listen_path.clone()
        };
        
        debug!("Adding route: {} -> {}", path, proxy.id);
        
        // Insert the proxy definition into the router
        if let Err(e) = router.insert(&path, proxy.clone()) {
            let error_msg = format!("Failed to insert route for proxy {}: {}", proxy.id, e);
            error!("{}", error_msg);
            return Err(GatewayError::RouterInsert(error_msg));
        }
    }
    
    Ok(router)
}
