use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, PrivatePkcs1KeyDer, PrivateSec1KeyDer}};
use rustls_pemfile::{certs, read_one, Item};

use crate::error::GatewayError;

pub fn load_tls_config(cert_path: &PathBuf, key_path: &PathBuf) -> Result<Arc<ServerConfig>, GatewayError> {
    // Load certificate chain
    let cert_file = File::open(cert_path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to open certificate file: {}", e)))?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs_bytes = certs(&mut cert_reader)
        .map_err(|_| GatewayError::TlsConfig("Failed to parse certificate".to_string()))?;
    
    // Convert certificates to the required format
    let cert_chain: Vec<CertificateDer<'static>> = certs_bytes
        .into_iter()
        .map(CertificateDer::from)
        .collect();

    // Load private key
    let key_file = File::open(key_path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to open private key file: {}", e)))?;
    let mut key_reader = BufReader::new(key_file);
    
    let key_item = read_one(&mut key_reader)
        .map_err(|_| GatewayError::TlsConfig("Failed to read private key".to_string()))?
        .ok_or_else(|| GatewayError::TlsConfig("No private key found".to_string()))?;

    let private_key = match key_item {
        Item::PKCS8Key(key) => PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)),
        Item::RSAKey(key) => PrivateKeyDer::Pkcs1(PrivatePkcs1KeyDer::from(key)),
        Item::ECKey(key) => PrivateKeyDer::Sec1(PrivateSec1KeyDer::from(key)),
        _ => return Err(GatewayError::TlsConfig("Unsupported private key format".to_string())),
    };

    // Build TLS config
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            cert_chain,
            private_key,
        )
        .map_err(|e| GatewayError::Tls(e))?;

    Ok(Arc::new(config))
}
