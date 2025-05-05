// Re-export modules to provide a clean public API
pub mod config;
pub mod error;
pub mod proxy;
pub mod server;
pub mod tls;

// Re-export common types for convenient usage
pub use config::ServerConfig;
pub use error::GatewayError;
pub use proxy::{AppState, ProxyDefinition, load_proxy_config};
pub use server::{handle_request, run_server};
pub use tls::load_tls_config;
