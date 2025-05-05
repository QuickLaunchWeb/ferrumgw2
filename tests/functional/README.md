# Ferrum API Gateway Functional Tests

This directory contains end-to-end functional tests for the Ferrum API Gateway. The tests verify that the gateway can properly:

1. Handle HTTP and HTTPS traffic
2. Route requests to appropriate backend services
3. Handle TLS termination
4. Forward requests to both HTTP and HTTPS backends
5. Strip path prefixes correctly

## Test Components

- **Mock API servers**: Simple HTTP and HTTPS servers that return JSON responses
- **TLS certificate generation**: Self-signed certificates for HTTPS testing
- **Gateway configuration**: Custom proxy configuration for testing
- **Test script**: Automated Python tests to verify correct functionality

## Requirements

- Rust/Cargo
- Python 3
- Python `requests` module
- OpenSSL

## Running the Tests

To run all tests:

```bash
./run_tests.sh
```

To run specific test stages:

```bash
./run_tests.sh clean
./run_tests.sh certs
./run_tests.sh mock-apis
./run_tests.sh run-gateway
./run_tests.sh test
```

## Test Flow

1. Clean up any previous test artifacts and processes
2. Generate TLS certificates if they don't exist
3. Start HTTP mock API servers on ports 8091 and 8092
4. Start HTTPS mock API servers on ports 8493 and 8494
5. Generate test proxy configuration for the gateway
6. Start the API Gateway with the test configuration
7. Run the test script to verify all endpoints work correctly

## Port Configuration

- **HTTP Mock APIs**: 8091, 8092
- **HTTPS Mock APIs**: 8493, 8494
- **Gateway HTTP**: 8081
- **Gateway HTTPS**: 8444

## Extending the Tests

To add more test cases, modify:

1. `mock_https_server.py` for more complex backend behavior
2. `test_proxies.yaml` to add more routing rules
3. The test script in `run_test_script` to add more endpoint tests

## Troubleshooting

If tests fail, check:

1. All services are running (use `ps aux | grep python` and `ps aux | grep cargo`)
2. Certificate paths are correct
3. Gateway logs for any routing errors
4. Network port availability (ensure no port conflicts)
