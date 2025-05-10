use std::sync::Arc;
use hyper::{Body, Request, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_rustls::HttpsConnectorBuilder;
use matchit::Router;
use std::net::SocketAddr;
use std::convert::Infallible;
use rust_api_gateway::{AppState, ProxyDefinition, handle_request};
use tokio::sync::oneshot;

// A mock backend server for testing
async fn run_test_backend(addr: SocketAddr, response_text: String, shutdown_rx: oneshot::Receiver<()>) {
    let service = make_service_fn(move |_| {
        let response_text = response_text.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |_| {
                let response_text = response_text.clone();
                async move {
                    Ok::<_, Infallible>(
                        Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(response_text))
                            .unwrap()
                    )
                }
            }))
        }
    });

    let server = Server::bind(&addr)
        .serve(service);

    let server_with_shutdown = server.with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });

    if let Err(e) = server_with_shutdown.await {
        eprintln!("Server error: {}", e);
    }
}

#[tokio::test]
async fn test_handle_request_for_known_route() {
    // Start a test backend server
    let backend_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let backend_response = "Hello from backend!".to_string();
    
    // Spawn the test backend server
    tokio::spawn(run_test_backend(backend_addr, backend_response.clone(), shutdown_rx));
    
    // Give the server a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Create a test router
    let mut router = Router::<ProxyDefinition>::new();
    
    let proxy = ProxyDefinition {
        id: "test-api".to_string(),
        name: "Test API".to_string(),
        listen_path: "/api".to_string(),
        backend_protocol: "http".to_string(),
        backend_host: "127.0.0.1".to_string(),
        backend_port: 8080,
        backend_path: "/".to_string(),
        strip_listen_path: false,
        preserve_host_header: false,
        backend_connect_timeout_ms: 3000,
        backend_read_timeout_ms: 30000,
        backend_write_timeout_ms: 30000,
        skip_certificate_verification: false,
    };
    
    router.insert("/api", proxy).unwrap();
    
    // Create HTTP and HTTPS clients for the tests
    let http_client = hyper::Client::new();
    
    // Create the HTTPS connector using rustls
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
        
    let https_client = hyper::Client::builder().build(https_connector);
    
    // Create app state with the router and clients
    let app_state = Arc::new(AppState { 
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    // Create a test request
    let request = Request::builder()
        .uri("http://localhost:3000/api")
        .body(Body::empty())
        .unwrap();
    
    // Process the request
    let response = handle_request(request, app_state).await.unwrap();
    
    // Check response status
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get response body
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify the response contains the expected text from our backend
    assert_eq!(body_str, backend_response);
    
    // Shutdown the test backend
    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn test_handle_request_for_unknown_route() {
    // Create an empty router
    let router = Router::<ProxyDefinition>::new();
    
    // Create HTTP and HTTPS clients for the tests
    let http_client = hyper::Client::new();
    
    // Create the HTTPS connector using rustls
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
        
    let https_client = hyper::Client::builder().build(https_connector);
    
    // Create app state with the router and clients
    let app_state = Arc::new(AppState { 
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    // Create a test request for a path that doesn't exist in the router
    let request = Request::builder()
        .uri("http://localhost:3000/unknown")
        .body(Body::empty())
        .unwrap();
    
    // Process the request
    let response = handle_request(request, app_state).await.unwrap();
    
    // Check that we get a 404 response
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Get response body
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify the response contains the expected text
    assert!(body_str.contains("Route Not Found"));
}

#[tokio::test]
async fn test_path_rewriting_with_strip_listen_path() {
    // Start a test backend server
    let backend_addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let backend_response = "Rewrite path test response".to_string();
    
    // Spawn the test backend server
    tokio::spawn(run_test_backend(backend_addr, backend_response.clone(), shutdown_rx));
    
    // Give the server a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Create a test router
    let mut router = Router::<ProxyDefinition>::new();
    
    let proxy = ProxyDefinition {
        id: "rewrite-api".to_string(),
        name: "Rewrite API Test".to_string(),
        listen_path: "/rewrite".to_string(),
        backend_protocol: "http".to_string(),
        backend_host: "127.0.0.1".to_string(),
        backend_port: 8081,
        backend_path: "/backend".to_string(),
        strip_listen_path: true,
        preserve_host_header: false,
        backend_connect_timeout_ms: 3000,
        backend_read_timeout_ms: 30000,
        backend_write_timeout_ms: 30000,
        skip_certificate_verification: false,
    };
    
    // Use /rewrite/:path to match any path that starts with /rewrite
    // In matchit, we need to use named parameters (with :) for wildcards
    router.insert("/rewrite/:path", proxy).unwrap();
    
    // Create HTTP and HTTPS clients for the tests
    let http_client = hyper::Client::new();
    
    // Create the HTTPS connector using rustls
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
        
    let https_client = hyper::Client::builder().build(https_connector);
    
    // Create app state with the router and clients
    let app_state = Arc::new(AppState { 
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    // Create a test request to /rewrite/test which should be forwarded to /backend/test
    let request = Request::builder()
        .uri("http://localhost:3000/rewrite/test")
        .body(Body::empty())
        .unwrap();
    
    // Process the request
    let response = handle_request(request, app_state).await.unwrap();
    
    // Check response status
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check for the X-Proxy-Id header first, before consuming the response
    assert_eq!(
        response.headers().get("x-proxy-id").unwrap().to_str().unwrap(),
        "rewrite-api"
    );
    
    // Get response body - this consumes the response
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify the response contains the expected text from our backend
    assert_eq!(body_str, backend_response);
    
    // Shutdown the test backend
    let _ = shutdown_tx.send(());
}

#[tokio::test]
#[ignore] // Requires internet connection - we'll run this manually
async fn test_https_backend() {
    // Create a test router
    let mut router = Router::<ProxyDefinition>::new();
    
    let proxy = ProxyDefinition {
        id: "https-test".to_string(),
        name: "HTTPS Test Service".to_string(),
        listen_path: "/https-test".to_string(),
        backend_protocol: "https".to_string(),
        backend_host: "httpbin.org".to_string(),
        backend_port: 443,
        backend_path: "/get".to_string(),
        strip_listen_path: true,
        preserve_host_header: false,
        backend_connect_timeout_ms: 5000,
        backend_read_timeout_ms: 30000,
        backend_write_timeout_ms: 30000,
        skip_certificate_verification: false,
    };
    
    router.insert("/https-test", proxy).unwrap();
    
    // Create HTTP and HTTPS clients for the tests
    let http_client = hyper::Client::new();
    
    // Create the HTTPS connector using rustls
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
        
    let https_client = hyper::Client::builder().build(https_connector);
    
    // Create app state with the router and clients
    let app_state = Arc::new(AppState { 
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    // Create a test request
    let request = Request::builder()
        .uri("http://localhost:3000/https-test")
        .body(Body::empty())
        .unwrap();
    
    // Process the request
    let response = handle_request(request, app_state).await.unwrap();
    
    // Check response status
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check for the X-Proxy-Id header first, before consuming the response
    assert_eq!(
        response.headers().get("x-proxy-id").unwrap().to_str().unwrap(),
        "https-test"
    );
    
    // Get response body - this consumes the response
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify the response contains the expected text from httpbin.org
    assert!(body_str.contains("httpbin.org"));
}
