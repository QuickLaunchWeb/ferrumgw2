#!/usr/bin/env python3
import http.server
import ssl
import sys
import json
import os
from http import HTTPStatus

class MockHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        # Map port numbers to expected service names
        port = int(sys.argv[1])
        service_name = "unknown"
        if port == 8493:
            service_name = "mock-https-1"
        elif port == 8494:
            service_name = "mock-https-2"
        
        response = {
            "service": service_name,
            "status": "ok",
            "port": port,
            "path": self.path
        }
        response_data = json.dumps(response).encode("utf-8")
        
        # Set proper headers with content length
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-type", "application/json")
        self.send_header("Content-Length", str(len(response_data)))
        self.send_header("Connection", "close")
        self.end_headers()
        
        # Send response body
        self.wfile.write(response_data)
        print(f"Handled GET request for {self.path}")

    def do_POST(self):
        # Read request body
        content_length = int(self.headers["Content-Length"]) if "Content-Length" in self.headers else 0
        post_data = self.rfile.read(content_length).decode("utf-8") if content_length > 0 else ""
        
        # Map port numbers to expected service names
        port = int(sys.argv[1])
        service_name = "unknown"
        if port == 8493:
            service_name = "mock-https-1"
        elif port == 8494:
            service_name = "mock-https-2"
        
        response = {
            "service": service_name,
            "status": "ok",
            "port": port,
            "path": self.path,
            "data": post_data
        }
        response_data = json.dumps(response).encode("utf-8")
        
        # Set proper headers with content length
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-type", "application/json")
        self.send_header("Content-Length", str(len(response_data)))
        self.send_header("Connection", "close")
        self.end_headers()
        
        # Send response body
        self.wfile.write(response_data)
        print(f"Handled POST request for {self.path} with data: {post_data}")

    def log_message(self, format, *args):
        # Override to provide more detailed logging
        sys.stderr.write(f"HTTPS Server ({sys.argv[1]}): {format % args}\n")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 mock_https_server.py <port>")
        sys.exit(1)

    port = int(sys.argv[1])
    server_address = ("localhost", port)
    
    # Find the project root directory
    current_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Go up to the root directory
    # For tests/functional/mock_apis/mock_https_server.py, we need to go up 3 levels
    root_dir = os.path.abspath(os.path.join(current_dir, '..', '..', '..'))
    if os.path.basename(root_dir) != 'ferrumgw2':
        # Alternative approach if the above doesn't work
        root_dir = os.path.abspath(os.path.join(current_dir, '..', '..'))
    
    # Set certificate paths
    cert_dir = os.path.join(root_dir, "certs")
    cert_path = os.path.join(cert_dir, "cert.pem")
    key_path = os.path.join(cert_dir, "key.pem")
    
    # Verify certificate files exist
    if not os.path.exists(cert_path) or not os.path.exists(key_path):
        print(f"Error: Certificate files not found at {cert_path} and {key_path}")
        # Try direct path
        cert_path = "/Users/jeremyjustus/workspace/ferrumgw2/certs/cert.pem"
        key_path = "/Users/jeremyjustus/workspace/ferrumgw2/certs/key.pem"
        if not os.path.exists(cert_path) or not os.path.exists(key_path):
            print(f"Error: Certificate files still not found at {cert_path} and {key_path}")
            sys.exit(1)
    
    print(f"Starting HTTPS server on port {port}...")
    print(f"Using certificates at {cert_path} and {key_path}")
    
    try:
        # Create HTTPS server with improved socket handling
        httpd = http.server.ThreadingHTTPServer(server_address, MockHandler)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(cert_path, key_path)
        httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
        print(f"HTTPS Server running on https://localhost:{port}")
        httpd.serve_forever()
    except Exception as e:
        print(f"Error starting HTTPS server on port {port}: {e}")
        sys.exit(1)
