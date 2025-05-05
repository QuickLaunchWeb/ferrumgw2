use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tracing::debug;

use crate::error::GatewayError;

/// Load TLS configuration for HTTPS server
///
/// This function reads the certificate chain and private key from PEM files
/// and constructs a rustls::ServerConfig for use with the HTTPS server.
/// 
/// # Arguments
/// * `cert_path` - Path to the certificate chain PEM file
/// * `key_path` - Path to the private key PEM file
///
/// # Returns
/// * `Result<Arc<ServerConfig>, GatewayError>` - The server configuration or an error
pub fn load_tls_config(cert_path: &PathBuf, key_path: &PathBuf) -> Result<Arc<ServerConfig>, GatewayError> {
    // Read certificate chain from file
    let cert_file = File::open(cert_path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to open certificate file: {}", e)))?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = certs(&mut cert_reader)
        .map_err(|_| GatewayError::TlsConfig("Failed to parse certificate".to_string()))?
        .into_iter()
        .map(Certificate)
        .collect();
    
    // Read private key from file - try PKCS8 format first, then RSA if that fails
    let key_file = File::open(key_path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to open private key file: {}", e)))?;
    let mut key_reader = BufReader::new(key_file);
    
    // Try to read PKCS8 private key
    let mut private_keys = pkcs8_private_keys(&mut key_reader)
        .map_err(|_| GatewayError::TlsConfig("Failed to parse PKCS8 private key".to_string()))?;
    
    // If no PKCS8 keys found, try RSA format
    if private_keys.is_empty() {
        // We need to re-open the file as the buffer has been consumed
        let key_file = File::open(key_path)
            .map_err(|e| GatewayError::TlsConfig(format!("Failed to reopen private key file: {}", e)))?;
        let mut key_reader = BufReader::new(key_file);
        
        private_keys = rsa_private_keys(&mut key_reader)
            .map_err(|_| GatewayError::TlsConfig("Failed to parse RSA private key".to_string()))?;
    }
    
    // Ensure we have at least one private key
    if private_keys.is_empty() {
        return Err(GatewayError::TlsConfig("No private keys found in key file".to_string()));
    }
    
    // Use the first private key found
    let private_key = PrivateKey(private_keys.remove(0));
    
    // Build TLS configuration
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .map_err(|e| GatewayError::TlsConfig(format!("TLS config error: {}", e)))?;
    
    // Enable HTTP/2 and HTTP/1.1 ALPN
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    
    debug!("TLS configuration loaded successfully");
    
    Ok(Arc::new(config))
}
