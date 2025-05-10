use std::sync::Arc;
use hyper_util::client::legacy::connect::HttpConnector;
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::DigitallySignedStruct;
use rustls::Error;
use rustls::SignatureScheme;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use crate::error::GatewayError;
use serde::Deserialize;
use std::fs;
use serde_yaml;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub id: String,
    pub listen_path: String,
    pub backend_protocol: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub backend_path: String,
    pub strip_listen_path: bool,
    pub preserve_host_header: bool,
    pub backend_connect_timeout_ms: u64,
    pub backend_read_timeout_ms: u64,
    pub backend_write_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyConfig {
    pub id: String,
    pub listen_path: String,
    pub backend_protocol: String,
    pub backend_host: String,
    pub backend_port: u16,
    pub backend_path: String,
    pub strip_listen_path: bool,
    pub preserve_host_header: bool,
    pub backend_connect_timeout_ms: u64,
    pub backend_read_timeout_ms: u64,
    pub backend_write_timeout_ms: u64,
}

#[derive(Debug)]
pub struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}

pub fn build_https_client(skip_verification: bool) -> Result<HttpsConnector<HttpConnector>, GatewayError> {
    let root_store = rustls::RootCertStore::empty();
    
    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    if skip_verification {
        config.dangerous().set_certificate_verifier(Arc::new(NoCertificateVerification {}));
    }

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(config)
        .https_or_http()
        .enable_http1()
        .build();
    
    Ok(https)
}

pub fn load_proxy_config(path: &Path) -> Result<Vec<Proxy>, GatewayError> {
    let config_file = fs::read_to_string(path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to read proxy config: {}", e)))?;
    
    let proxies: Vec<Proxy> = serde_yaml::from_str(&config_file)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to parse proxy config: {}", e)))?;
    
    Ok(proxies)
}

pub fn load_proxy_config_from_path(path: &Path) -> Result<Vec<ProxyConfig>, GatewayError> {
    let config_file = fs::read_to_string(path)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to read proxy config: {}", e)))?;
    
    let proxies: Vec<ProxyConfig> = serde_yaml::from_str(&config_file)
        .map_err(|e| GatewayError::TlsConfig(format!("Failed to parse proxy config: {}", e)))?;
    
    Ok(proxies)
}