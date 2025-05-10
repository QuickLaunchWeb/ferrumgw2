use tracing::info;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_basic_logging() {
    info!("This is a test log message");
    assert!(logs_contain("This is a test log message"));
}

#[tokio::test]
#[traced_test(level = "debug")]
async fn test_structured_logging() {
    tracing::debug!("Processing request");
    tracing::info!("Request completed");
    tracing::warn!("Resource not found");
    tracing::error!("Failed to connect to database");
    
    assert!(logs_contain("Processing request"));
    assert!(logs_contain("Request completed"));
    assert!(logs_contain("Resource not found"));
    assert!(logs_contain("Failed to connect to database"));
}
