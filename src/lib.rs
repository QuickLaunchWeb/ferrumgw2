// Re-export modules to provide a clean public API
pub mod config;
pub mod error;
pub mod proxy;
pub mod server;
pub mod tls;

// Re-export common types for convenient usage
pub use server::{handle_request, serve};
pub use error::GatewayError;
pub use proxy::{Proxy, load_proxy_config};

use std::sync::Arc;

pub struct AppState {
    pub router: Arc<matchit::Router<proxy::Proxy>>,
    pub http_client: hyper_util::client::legacy::Client<hyper_util::client::legacy::connect::HttpConnector, hyper::body::Incoming>,
    pub https_client: hyper_util::client::legacy::Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, hyper::body::Incoming>,
}
