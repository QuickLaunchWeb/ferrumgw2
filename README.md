# Ferrum API Gateway

A high-performance API Gateway and Reverse Proxy written in Rust, featuring robust TLS support, flexible routing, and configurable proxying capabilities.

## Overview

Ferrum is designed to be a lightweight yet powerful API Gateway that can route client requests to various backend services based on URL paths. It supports both HTTP and HTTPS, with TLS termination and certificate validation options.

## Features

- **HTTP/HTTPS Support**: Listen on both HTTP and HTTPS endpoints simultaneously
- **TLS Termination**: Handle HTTPS connections with proper certificate management
- **Flexible Routing**: Route requests based on URL paths using a Radix Tree for efficient matching
- **Dynamic Proxy Configuration**: Configure proxy destinations via YAML files
- **Path Rewriting**: Strip path prefixes for cleaner backend URLs
- **Host Header Preservation**: Option to preserve the original host header
- **Certificate Verification Control**: Skip certificate verification for development environments
- **Connection Timeout Management**: Configurable timeouts for backend connections
- **HTTP/2 Support**: Modern protocol support for improved performance

## Architecture

The project follows a modular architecture with these main components:

- **Server**: HTTP/HTTPS server implementation for handling client requests
- **Router**: Path-based routing using the matchit library (Radix Tree implementation)
- **Proxy**: Request forwarding to appropriate backend services
- **Config**: Configuration loading from environment variables and YAML files
- **Error Handling**: Comprehensive error management with thiserror

## Requirements

- Rust (latest stable version recommended)
- OpenSSL for TLS certificate generation (for development/testing)
- Python 3 (for running functional tests)

## Running the API Gateway

### Environment Setup

Create a `.env` file in the project root with the following variables:

```
PROXY_CONFIG_PATH=config/proxies.yaml
HTTP_PORT=8081
HTTPS_PORT=8444
CERT_PATH=certs/cert.pem
KEY_PATH=certs/key.pem
LOG_LEVEL=info
```

### Building and Running

```bash
# Build the project
cargo build --release

# Run the API Gateway
cargo run --release

# With specific log level
RUST_LOG=debug cargo run
```

## Configuration

### Environment Variables

| Variable          | Description                           | Default     |
|-------------------|---------------------------------------|-------------|
| PROXY_CONFIG_PATH | Path to proxy configuration YAML file | config/proxies.yaml |
| HTTP_PORT         | Port for HTTP server                  | 8081        |
| HTTPS_PORT        | Port for HTTPS server                 | 8444        |
| CERT_PATH         | Path to TLS certificate               | certs/cert.pem |
| KEY_PATH          | Path to TLS private key               | certs/key.pem |
| LOG_LEVEL         | Logging level                         | info        |

### Proxy Configuration

Proxies are configured via a YAML file. Example:

```yaml
proxies:
  - id: service-1
    name: Service One
    listen_path: /api/service1
    backend_protocol: http
    backend_host: localhost
    backend_port: 8091
    backend_path: /
    strip_listen_path: true
    preserve_host_header: false
    skip_certificate_verification: false
    backend_connect_timeout_ms: 3000
    backend_read_timeout_ms: 30000
    backend_write_timeout_ms: 30000
  
  - id: secure-service
    name: Secure Service
    listen_path: /api/secure
    backend_protocol: https
    backend_host: example.com
    backend_port: 443
    backend_path: /v1
    strip_listen_path: true
    preserve_host_header: true
    skip_certificate_verification: true
    backend_connect_timeout_ms: 5000
    backend_read_timeout_ms: 60000
    backend_write_timeout_ms: 30000
```

#### Proxy Configuration Options

| Option                      | Description                                                  | Default |
|-----------------------------|--------------------------------------------------------------|---------|
| id                          | Unique identifier for the proxy                              | (required) |
| name                        | Display name for the proxy                                   | (required) |
| listen_path                 | URL path to match incoming requests                          | (required) |
| backend_protocol            | Protocol for backend (http/https)                            | (required) |
| backend_host                | Hostname of the backend service                              | (required) |
| backend_port                | Port of the backend service                                  | 80      |
| backend_path                | Base path to prepend to incoming paths                       | /       |
| strip_listen_path           | Whether to strip the listen_path before proxying             | false   |
| preserve_host_header        | Whether to keep original Host header                         | false   |
| skip_certificate_verification | Skip TLS certificate verification                          | false   |
| backend_connect_timeout_ms  | Connection timeout in milliseconds                           | 3000    |
| backend_read_timeout_ms     | Read timeout in milliseconds                                 | 30000   |
| backend_write_timeout_ms    | Write timeout in milliseconds                                | 30000   |

## Testing

### Unit Tests

Run unit tests with:

```bash
cargo test
```

### Functional Tests

The project includes a comprehensive functional test suite that verifies end-to-end API Gateway functionality with both HTTP and HTTPS backends.

To run the functional tests:

```bash
cd tests/functional
./run_tests.sh
```

#### Test Configuration

The functional tests:
- Generate self-signed certificates
- Start mock HTTP and HTTPS servers
- Configure and start the API Gateway
- Send requests through the gateway to verify proper routing and responses

To clean up test artifacts:

```bash
cd tests/functional
./run_tests.sh clean
```

## Development Guidelines

- Keep the codebase modular and maintainable
- Add proper documentation for all public interfaces
- Write tests for new functionality
- Use the existing error handling framework for consistent error management

## License

MIT
