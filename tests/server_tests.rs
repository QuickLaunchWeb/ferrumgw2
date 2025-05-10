use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use hyper_util::rt::TokioExecutor;
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;
use http_body_util::{Full, BodyExt};
use matchit::Router;
use rust_api_gateway::{AppState, Proxy, handle_request};

async fn run_test_backend(addr: SocketAddr, response: String) -> tokio::task::JoinHandle<()> {
    let response_clone = response.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);

        let service = hyper::service::service_fn(move |_| {
            let response = response_clone.clone();
            async move {
                Ok::<_, hyper::Error>(
                    Response::new(Full::new(Bytes::from(response)))
                )
            }
        });

        Builder::new(TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .unwrap();
    })
}

async fn setup_test() -> (Arc<AppState>, tokio::task::JoinHandle<()>) {
    let backend_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let backend_response = "message".to_string();
    
    let backend_handle = run_test_backend(backend_addr, backend_response.clone()).await;
    
    let mut router = Router::new();
    
    let proxy = Proxy {
        id: "test-proxy".to_string(),
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
    };
    
    router.insert("/api", proxy).unwrap();
    
    let http_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
    let https_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
        .build(hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_only()
            .enable_http1()
            .build());
    
    let app_state = Arc::new(AppState {
        router: Arc::new(router),
        http_client,
        https_client,
    });
    
    (app_state, backend_handle)
}

#[tokio::test]
async fn test_handle_request_for_known_route() {
    // Setup test environment
    let (app_state, backend_handle) = setup_test().await;
    
    // Create request with Full<Bytes> body
    let request = Request::builder()
        .uri("http://localhost:8081/api/test")
        .body(Full::new(Bytes::new()))
        .unwrap();
    
    // Handle request
    let response = handle_request(request, app_state.clone()).await.unwrap();
    
    // Verify response
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body, "message");
    
    backend_handle.abort();
}