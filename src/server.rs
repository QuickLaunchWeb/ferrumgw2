// In server.rs
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Request, Response, StatusCode};
use hyper::body::Body;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper_util::server::conn::auto::Builder;
use crate::error::GatewayError;
use bytes::Bytes;
use http_body_util::Full;
use tracing::error;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, Infallible>;

pub async fn handle_request<B>(_req: Request<B>, _state: Arc<crate::AppState>) -> Result<Response<Full<Bytes>>, GatewayError>
where
    B: Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let body = Full::new(Bytes::from("message"));
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(body)
        .unwrap();

    Ok(response)
}

pub async fn serve(addr: SocketAddr, app_state: Arc<crate::AppState>) -> Result<(), GatewayError> {
    let listener = TcpListener::bind(addr).await?;
    
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = app_state.clone();
        
        tokio::task::spawn(async move {
            if let Err(err) = Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service_fn(move |req| {
                    handle_request(req, state.clone())
                }))
                .await
            {
                error!(%err, "Failed to serve connection");
            }
        });
    }
}