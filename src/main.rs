use std::net::SocketAddr;
use std::sync::Arc;

use hyper_util::rt::TokioExecutor;
use rust_api_gateway::{serve, AppState};
use rust_api_gateway::config::load_config;
use rust_api_gateway::proxy::load_proxy_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration
    let config = load_config()?;
    
    // Load proxy configurations
    let proxies = load_proxy_config(&config.proxy_config_path)?;
    
    // Create router and insert proxies
    let mut router = matchit::Router::new();
    for proxy in proxies {
        let path = proxy.listen_path.clone();
        router.insert(&path, proxy).unwrap();
    }
    
    // Create HTTP clients
    let http_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
    let https_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
        .build(hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_only()
            .enable_http1()
            .build());
    
    // Create app state
    let app_state = Arc::new(AppState {
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    serve(addr, app_state).await?;
    
    Ok(())
}
