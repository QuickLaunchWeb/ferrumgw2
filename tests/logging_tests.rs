use tracing::info;
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_basic_logging() {
    // Create a log message at info level
    info!("This is a test log message");
    
    // Verify log was captured
    assert!(logs_contain("This is a test log message"));
}

#[tokio::test]
#[traced_test(level = "debug")]
async fn test_structured_logging() {
    // Create structured logs with different levels
    tracing::trace!(target: "app_events", user_id = 1, "User logged in");
    tracing::debug!(request_id = "req-123", "Processing request");
    tracing::info!(status = "success", duration = 127, "Request completed");
    tracing::warn!(error_code = 404, "Resource not found");
    tracing::error!(error_type = "database_error", "Failed to connect to database");
    
    // Verify debug logs (and higher levels) are captured
    assert!(logs_contain("Processing request"));
    assert!(logs_contain("Request completed"));
    assert!(logs_contain("Resource not found"));
    assert!(logs_contain("Failed to connect to database"));
    
    // Trace level logs shouldn't be captured with debug level filter
    assert!(!logs_contain("User logged in"));
}
