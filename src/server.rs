use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use hyper::{Body, Request, Response, Server, StatusCode, Uri};
use hyper::http::HeaderValue;
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, service_fn};
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

use crate::error::GatewayError;
use crate::proxy::AppState;
use crate::tls::load_tls_config;

/// Handler for incoming requests
pub async fn handle_request(req: Request<Body>, state: Arc<AppState>) 
    -> Result<Response<Body>, Infallible> 
{
    // Create a span for this request
    let span = tracing::info_span!(
        "request", 
        method = %req.method(), 
        path = %req.uri().path(), 
        version = ?req.version()
    );
    
    let _guard = span.enter();
    
    debug!("Received request: {} {}", req.method(), req.uri().path());
    
    // Try to match the request path in the router
    match state.router.at(req.uri().path()) {
        Ok(matched) => {
            // Get the matched proxy definition
            let proxy = matched.value;
            
            debug!("Route matched: id={}, name={}", proxy.id, proxy.name);
            
            // Construct target URI
            let path_to_forward = if proxy.strip_listen_path {
                // Remove the listen_path from the beginning of the request path
                let request_path = req.uri().path();
                if request_path.starts_with(&proxy.listen_path) {
                    // If the listen_path is "/api" and request is "/api/users", the forwarded path becomes "/users"
                    let remaining_path = &request_path[proxy.listen_path.len()..];
                    if remaining_path.is_empty() {
                        proxy.backend_path.clone()
                    } else if proxy.backend_path.ends_with('/') && remaining_path.starts_with('/') {
                        // Avoid double slashes
                        format!("{}{}", &proxy.backend_path[..proxy.backend_path.len()-1], remaining_path)
                    } else if !proxy.backend_path.ends_with('/') && !remaining_path.starts_with('/') {
                        // Add slash between paths
                        format!("{}/{}", proxy.backend_path, remaining_path)
                    } else {
                        format!("{}{}", proxy.backend_path, remaining_path)
                    }
                } else {
                    // This shouldn't happen if the router matched correctly
                    format!("{}{}", proxy.backend_path, req.uri().path())
                }
            } else {
                // Forward the full path
                format!("{}{}", proxy.backend_path, req.uri().path())
            };
            
            // Create query string if present
            let query_string = req.uri().query().map_or_else(String::new, |q| format!("?{}", q));
            
            // Combine to form the complete URI
            let uri_string = format!(
                "{}://{}:{}{}{}",
                proxy.backend_protocol,
                proxy.backend_host,
                proxy.backend_port,
                path_to_forward,
                query_string
            );
            
            debug!("Forwarding request to: {}", uri_string);
            
            let uri = match uri_string.parse::<Uri>() {
                Ok(uri) => uri,
                Err(e) => {
                    error!("Failed to parse target URI: {}", e);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to construct target URI"))
                        .unwrap());
                }
            };
            
            // Build the request to forward
            let mut builder = Request::builder()
                .method(req.method())
                .uri(uri)
                .version(req.version());
            
            // Copy headers from the original request
            let headers = builder.headers_mut().unwrap();
            for (name, value) in req.headers() {
                // Skip connection-specific headers
                if name == "connection" || name == "keep-alive" || name == "proxy-connection" || 
                   name == "transfer-encoding" || name == "upgrade" {
                    continue;
                }
                
                // Handle host header based on configuration
                if name == "host" && !proxy.preserve_host_header {
                    // Don't forward the original host header if preserve_host_header is false
                    continue;
                }
                
                headers.insert(name, value.clone());
            }
            
            // If we didn't preserve the host header and it wasn't already set, set it to the backend host
            if !proxy.preserve_host_header && !headers.contains_key("host") {
                let host_value = format!("{}:{}", proxy.backend_host, proxy.backend_port);
                headers.insert("host", HeaderValue::from_str(&host_value).unwrap_or_else(|_| {
                    HeaderValue::from_static("unknown")
                }));
            }
            
            // Set X-Forwarded headers
            if let Some(client_ip) = req.headers().get("x-forwarded-for") {
                // Append to existing X-Forwarded-For header
                let forwarded_for = format!("{}, 127.0.0.1", client_ip.to_str().unwrap_or(""));
                headers.insert("x-forwarded-for", HeaderValue::from_str(&forwarded_for).unwrap_or_else(|_| {
                    HeaderValue::from_static("127.0.0.1")
                }));
            } else {
                // Create new X-Forwarded-For with client IP
                headers.insert("x-forwarded-for", HeaderValue::from_static("127.0.0.1"));
            }
            
            // Add X-Forwarded-Proto
            headers.insert("x-forwarded-proto", HeaderValue::from_static("http"));
            
            // Add X-Forwarded-Host if not already set
            if !headers.contains_key("x-forwarded-host") {
                if let Some(host) = req.headers().get("host") {
                    headers.insert("x-forwarded-host", host.clone());
                }
            }
            
            // Build the final request with the original body
            let forwarded_req = match builder.body(req.into_body()) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to build forwarded request: {}", e);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to build forwarded request"))
                        .unwrap());
                }
            };
            
            // Choose the appropriate client based on the backend protocol
            let response_result = match proxy.backend_protocol.as_str() {
                "https" => {
                    debug!("Using HTTPS client for backend request");
                    state.https_client.request(forwarded_req).await
                },
                _ => {
                    debug!("Using HTTP client for backend request");
                    state.http_client.request(forwarded_req).await
                }
            };
            
            // Forward the request to the backend
            match response_result {
                Ok(backend_response) => {
                    debug!(
                        "Received response from backend: status={}",
                        backend_response.status()
                    );
                    
                    // Copy the response status and body
                    let mut response_builder = Response::builder()
                        .status(backend_response.status());
                    
                    // Copy headers from the backend response
                    {
                        let headers = response_builder.headers_mut().unwrap();
                        for (name, value) in backend_response.headers() {
                            // Skip connection-specific headers
                            if name == "connection" || name == "keep-alive" || name == "transfer-encoding" {
                                continue;
                            }
                            headers.insert(name, value.clone());
                        }
                        
                        // Add X-Proxy-Id header to identify which proxy processed the request
                        headers.insert("x-proxy-id", HeaderValue::from_str(&proxy.id).unwrap_or_else(|_| {
                            HeaderValue::from_static("unknown")
                        }));
                    }
                    
                    // Finalize the response with the backend response body
                    Ok(response_builder
                        .body(backend_response.into_body())
                        .unwrap_or_else(|_| {
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to construct response"))
                                .unwrap()
                        }))
                },
                Err(e) => {
                    error!("Failed to forward request to backend: {}", e);
                    
                    // Return a 502 Bad Gateway response
                    Ok(Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::from(format!("Failed to forward request to backend: {}", e)))
                        .unwrap())
                }
            }
        },
        Err(_) => {
            // No matching route found
            let error_msg = format!("Route not found for path: {}", req.uri().path());
            warn!("{}", error_msg);
            
            // Return a 404 Not Found response
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Route Not Found"))
                .unwrap())
        }
    }
}

/// Run the server with HTTP and optional HTTPS support
pub async fn run_server(addr: SocketAddr, app_state: Arc<AppState>, 
                        tls_cert_path: Option<PathBuf>, tls_key_path: Option<PathBuf>, 
                        https_port: u16) -> Result<(), GatewayError> {
    info!("Starting HTTP server on port {}", addr.port());
    
    // Clone the app state for use in the service factory
    let state = app_state.clone();
    
    // Create service function for handling requests - will be used for both HTTP and HTTPS
    let make_svc = make_service_fn(move |conn: &hyper::server::conn::AddrStream| {
        // Log connection details
        let remote_addr = conn.remote_addr();
        info!("New connection from: {}", remote_addr);
        
        // Clone the app state for each connection
        let state = state.clone();
        
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                // Clone the app state for each request
                let state = state.clone();
                
                // Log request details
                info!("Request from {}: {} {}", remote_addr, req.method(), req.uri());
                
                // Handle the request
                handle_request(req, state)
            }))
        }
    });
    
    // Build the HTTP server
    let http_server = Server::bind(&addr)
        .serve(make_svc.clone());
    
    // Create HTTPS server if TLS is configured
    let https_server_future = if let (Some(cert_path), Some(key_path)) = (tls_cert_path, tls_key_path) {
        info!("TLS certificates found, configuring HTTPS server");
        
        // Load TLS configuration
        let tls_config = load_tls_config(&cert_path, &key_path)?;
        let tls_acceptor = TlsAcceptor::from(tls_config);
        
        // Create HTTPS address
        let https_addr = SocketAddr::from(([0, 0, 0, 0], https_port));
        
        // Bind TCP listener for HTTPS
        let https_listener = TcpListener::bind(&https_addr).await
            .map_err(|e| GatewayError::Io(e))?;
        
        info!("HTTPS Server listening on {}", https_addr);
        
        // Clone app_state for HTTPS server
        let https_app_state = app_state.clone();
        
        // Spawn HTTPS server task
        let https_server = tokio::spawn(async move {
            loop {
                // Accept incoming connections
                let (tcp_stream, remote_addr) = match https_listener.accept().await {
                    Ok((stream, addr)) => (stream, addr),
                    Err(e) => {
                        error!("Failed to accept HTTPS connection: {}", e);
                        continue;
                    }
                };
                
                // Clone TLS acceptor for this connection
                let acceptor = tls_acceptor.clone();
                
                // Clone app state for this connection
                let conn_state = https_app_state.clone();
                
                // Spawn task to handle this connection
                tokio::spawn(async move {
                    // Perform TLS handshake
                    let tls_stream = match acceptor.accept(tcp_stream).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            warn!("TLS handshake failed from {}: {}", remote_addr, e);
                            return;
                        }
                    };
                    
                    info!("TLS handshake completed with {}", remote_addr);
                    
                    // Create a service for this specific connection
                    let service = service_fn(move |req| {
                        let req_state = conn_state.clone();
                        info!("HTTPS Request from {}: {} {}", remote_addr, req.method(), req.uri());
                        
                        // Handle the request
                        handle_request(req, req_state)
                    });
                    
                    // Create HTTP protocol handler with HTTP/2 support
                    let mut http = Http::new();
                    
                    // Support both HTTP/1.1 and HTTP/2, allowing HTTP/2 to be negotiated via ALPN
                    http.http2_only(false);
                    http.http2_initial_stream_window_size(1024 * 1024);  // 1MB
                    http.http2_initial_connection_window_size(1024 * 1024 * 10); // 10MB
                    http.http2_adaptive_window(true);
                    http.http2_keep_alive_interval(Some(std::time::Duration::from_secs(30)));
                    
                    // Serve connection
                    if let Err(e) = http.serve_connection(tls_stream, service).await {
                        warn!("Error serving TLS connection from {}: {}", remote_addr, e);
                    }
                });
            }
        });
        
        Some(https_server)
    } else {
        info!("TLS not configured, HTTPS server not starting");
        None
    };
    
    // Add graceful shutdown on Ctrl+C for HTTP server
    let http_server_with_shutdown = http_server.with_graceful_shutdown(async {
        match ctrl_c().await {
            Ok(()) => info!("Shutdown signal received, initiating graceful shutdown"),
            Err(err) => error!("Error listening for shutdown signal: {}", err),
        }
    });
    
    // Start the HTTP server
    info!("HTTP Server listening on {}", addr);
    
    // Wait for HTTP server to complete (this blocks until shutdown)
    if let Err(e) = http_server_with_shutdown.await {
        error!("HTTP Server error: {}", e);
    }
    
    // If HTTPS server is running, abort it when HTTP server is done
    if let Some(https_handle) = https_server_future {
        https_handle.abort();
        match https_handle.await {
            Ok(_) => info!("HTTPS server shutdown completed"),
            Err(e) => warn!("Error during HTTPS server shutdown: {}", e),
        }
    }
    
    info!("Server shutdown completed");
    
    Ok(())
}
