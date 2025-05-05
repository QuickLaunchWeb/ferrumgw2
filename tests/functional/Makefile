# Functional Tests Makefile for Ferrum API Gateway
# This Makefile manages end-to-end testing with TLS certificate generation,
# mock API servers, and API Gateway testing

# Configuration
SHELL := /bin/bash
.SHELLFLAGS := -e -o pipefail -c
.PHONY: all clean certs mock-apis run-gateway test

# Directories
ROOT_DIR := $(shell cd ../../ && pwd)
CERT_DIR := $(ROOT_DIR)/certs
MOCK_DIR := $(ROOT_DIR)/tests/functional/mock_apis
CONFIG_DIR := $(ROOT_DIR)/tests/functional/configs

# Ports
HTTP_PORT1 := 8091
HTTP_PORT2 := 8092
HTTPS_PORT1 := 8493
HTTPS_PORT2 := 8494
GATEWAY_HTTP_PORT := 8081
GATEWAY_HTTPS_PORT := 8444

# Certificate information
CERT_DAYS := 365
CERT_COUNTRY := US
CERT_STATE := California
CERT_LOCALITY := San Francisco
CERT_ORG := Ferrum
CERT_CN := localhost

# Default target
all: clean certs mock-apis run-gateway test

# Clean up generated files and processes
clean:
	@echo "Cleaning up previous test artifacts and processes..."
	# Kill all Python HTTP servers
	@pkill -f "python3 -m http.server" || true
	@sleep 1
	
	# Kill all Python HTTPS mock servers
	@pkill -f "python3 ./mock_https_server.py" || true
	@sleep 1
	
	# Kill all Rust API Gateway instances
	@pkill -f "cargo run --bin rust_api_gateway" || true
	@sleep 1
	
	# Kill any processes using the specific ports we need
	@echo "Releasing ports if in use..."
	@lsof -ti:$(HTTP_PORT1),$(HTTP_PORT2),$(HTTPS_PORT1),$(HTTPS_PORT2),$(GATEWAY_HTTP_PORT),$(GATEWAY_HTTPS_PORT) | xargs kill -9 2>/dev/null || true
	@sleep 2
	
	@mkdir -p $(CERT_DIR) $(MOCK_DIR) $(CONFIG_DIR)

# Generate certificates for TLS testing
certs:
	@echo "Generating TLS certificates..."
	@mkdir -p $(CERT_DIR)
	@if [ ! -f $(CERT_DIR)/key.pem ] || [ ! -f $(CERT_DIR)/cert.pem ]; then \
		openssl req -x509 -newkey rsa:4096 -nodes -keyout $(CERT_DIR)/key.pem -out $(CERT_DIR)/cert.pem \
			-days $(CERT_DAYS) -subj "/C=$(CERT_COUNTRY)/ST=$(CERT_STATE)/L=$(CERT_LOCALITY)/O=$(CERT_ORG)/CN=$(CERT_CN)" \
			-addext "subjectAltName = DNS:localhost,IP:127.0.0.1"; \
		echo "Certificates generated successfully."; \
	else \
		echo "Certificates already exist. Using existing certificates."; \
	fi

# Setup and run mock API servers
mock-apis: mock-http-apis mock-https-apis

# Create HTTP mock API servers
mock-http-apis:
	@echo "Setting up HTTP mock API servers..."
	@mkdir -p $(MOCK_DIR)
	@cp $(ROOT_DIR)/tests/functional/mock_http_server.py $(MOCK_DIR)/mock_http_server.py
	@chmod +x $(MOCK_DIR)/mock_http_server.py
	@cd $(MOCK_DIR) && python3 ./mock_http_server.py $(HTTP_PORT1) &
	@cd $(MOCK_DIR) && python3 ./mock_http_server.py $(HTTP_PORT2) &
	@echo "HTTP mock servers started on ports $(HTTP_PORT1) and $(HTTP_PORT2)"

# Create HTTPS mock API servers
mock-https-apis: certs
	@echo "Setting up HTTPS mock API servers..."
	@mkdir -p $(MOCK_DIR)
	@cp $(ROOT_DIR)/tests/functional/mock_https_server.py $(MOCK_DIR)/mock_https_server.py
	@chmod +x $(MOCK_DIR)/mock_https_server.py
	@cd $(MOCK_DIR) && python3 ./mock_https_server.py $(HTTPS_PORT1) &
	@cd $(MOCK_DIR) && python3 ./mock_https_server.py $(HTTPS_PORT2) &
	@echo "HTTPS mock servers started on ports $(HTTPS_PORT1) and $(HTTPS_PORT2)"
	@# Give the servers a moment to start up
	@sleep 2

# Generate test proxy configurations
setup-gateway-config:
	@echo "Setting up API Gateway test configurations..."
	@mkdir -p $(CONFIG_DIR)
	@echo "proxies:" > $(CONFIG_DIR)/test_proxies.yaml
	@echo "  - id: http-service-1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    name: HTTP Service 1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    listen_path: /api/http1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_protocol: http" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_host: localhost" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_port: $(HTTP_PORT1)" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_path: /" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    strip_listen_path: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    preserve_host_header: false" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_connect_timeout_ms: 3000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_read_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_write_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "  - id: http-service-2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    name: HTTP Service 2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    listen_path: /api/http2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_protocol: http" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_host: localhost" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_port: $(HTTP_PORT2)" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_path: /" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    strip_listen_path: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    preserve_host_header: false" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_connect_timeout_ms: 3000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_read_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_write_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "  - id: https-service-1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    name: HTTPS Service 1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    listen_path: /api/https1" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_protocol: https" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_host: localhost" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_port: $(HTTPS_PORT1)" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_path: /" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    strip_listen_path: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    preserve_host_header: false" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    skip_certificate_verification: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_connect_timeout_ms: 3000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_read_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_write_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "  - id: https-service-2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    name: HTTPS Service 2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    listen_path: /api/https2" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_protocol: https" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_host: localhost" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_port: $(HTTPS_PORT2)" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_path: /" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    strip_listen_path: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    preserve_host_header: false" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    skip_certificate_verification: true" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_connect_timeout_ms: 3000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_read_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "    backend_write_timeout_ms: 30000" >> $(CONFIG_DIR)/test_proxies.yaml
	@echo "PROXY_CONFIG_PATH=$(CONFIG_DIR)/test_proxies.yaml" > $(CONFIG_DIR)/test.env
	@echo "HTTP_PORT=$(GATEWAY_HTTP_PORT)" >> $(CONFIG_DIR)/test.env
	@echo "HTTPS_PORT=$(GATEWAY_HTTPS_PORT)" >> $(CONFIG_DIR)/test.env
	@echo "CERT_PATH=$(CERT_DIR)/cert.pem" >> $(CONFIG_DIR)/test.env
	@echo "KEY_PATH=$(CERT_DIR)/key.pem" >> $(CONFIG_DIR)/test.env
	@echo "LOG_LEVEL=debug" >> $(CONFIG_DIR)/test.env

# Run the API Gateway with test configuration
run-gateway: setup-gateway-config
	@echo "Starting API Gateway with test configuration..."
	@cd $(ROOT_DIR) && RUST_LOG=debug \
		PROXY_CONFIG_PATH=$(CONFIG_DIR)/test_proxies.yaml \
		HTTP_PORT=$(GATEWAY_HTTP_PORT) \
		HTTPS_PORT=$(GATEWAY_HTTPS_PORT) \
		CERT_PATH=$(CERT_DIR)/cert.pem \
		KEY_PATH=$(CERT_DIR)/key.pem \
		cargo run --bin rust_api_gateway &
	@echo "Waiting for API Gateway to start..."
	@sleep 3

# Run the functional tests against running services
test: run-test-script
	@echo "All functional tests completed."

# Create and run the test script
run-test-script:
	@echo "Running functional tests..."
	@mkdir -p $(MOCK_DIR)
	@cp $(ROOT_DIR)/tests/functional/run_tests.py $(MOCK_DIR)/run_tests.py
	@chmod +x $(MOCK_DIR)/run_tests.py
	@cd $(MOCK_DIR) && python3 -c "import sys; sys.path.append('/opt/homebrew/lib/python3.11/site-packages'); exec(open('./run_tests.py').read())" $(GATEWAY_HTTP_PORT) $(GATEWAY_HTTPS_PORT)
