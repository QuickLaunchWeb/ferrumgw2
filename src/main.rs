use std::net::SocketAddr;
use std::sync::Arc;

use dotenv::dotenv;
use tracing::{debug, info, trace, warn, error};
use tracing_subscriber::{EnvFilter, fmt};

// Import from our own crate
use rust_api_gateway::{
    ServerConfig,
    GatewayError,
    AppState,
    load_proxy_config,
    run_server
};

#[tokio::main]
async fn main() -> Result<(), GatewayError> {
    // Initialize tracing subscriber with env filter first
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)  // Enable targets in log messages
        .with_thread_ids(true)  // Show thread IDs
        .with_thread_names(true)  // Show thread names if available
        .with_file(true)  // Include file information
        .with_line_number(true)  // Include line numbers
        .init();
    
    // Output a few test log messages at different levels to verify logging is working
    tracing::trace!("TRACE level message - should appear with RUST_LOG=trace");
    tracing::debug!("DEBUG level message - should appear with RUST_LOG=debug");
    tracing::info!("INFO level message - should appear with RUST_LOG=info");
    tracing::warn!("WARN level message - should appear with RUST_LOG=warn");
    tracing::error!("ERROR level message - should appear with RUST_LOG=error");
    
    // Load .env file if present
    match dotenv() {
        Ok(_) => info!("Loaded environment from .env file"),
        Err(_) => info!("No .env file found or error loading it, using environment variables only"),
    }
    
    info!("Loading configuration from environment");
    
    // Load configuration, propagate errors with ?
    let config = ServerConfig::from_env()?;
    
    info!("Configuration loaded successfully");
    debug!("Server config: {:?}", config);
    
    // Log TLS configuration status
    if config.tls_cert_path.is_some() && config.tls_key_path.is_some() {
        info!(
            "TLS configuration found: cert={}, key={}", 
            config.tls_cert_path.as_ref().unwrap().display(),
            config.tls_key_path.as_ref().unwrap().display()
        );
    } else {
        info!("TLS configuration not found, HTTPS server will not start");
    }
    
    // Load proxy configuration
    info!("Loading proxy configuration from {}", config.proxy_config_path.display());
    let proxies = load_proxy_config(&config.proxy_config_path)?;
    
    // Log proxy configuration details
    info!("Loaded {} proxy definitions", proxies.len());
    for proxy in &proxies {
        debug!(
            "Registered proxy: id={}, name={}, listen_path={}, backend={}://{}:{}{}",
            proxy.id,
            proxy.name,
            proxy.listen_path,
            proxy.backend_protocol,
            proxy.backend_host,
            proxy.backend_port,
            proxy.backend_path
        );
    }
    
    // Build the application state with router
    info!("Building routing table");
    let app_state = Arc::new(AppState::new(&proxies)?);
    info!("Routing table built successfully with {} routes", proxies.len());
    
    // Set up the socket address from the configuration
    let addr = SocketAddr::from(([0, 0, 0, 0], config.http_port));
    
    // Run the HTTP server and HTTPS server if configured
    run_server(
        addr, 
        app_state, 
        config.tls_cert_path, 
        config.tls_key_path, 
        config.https_port
    ).await?;
    
    Ok(())
}
