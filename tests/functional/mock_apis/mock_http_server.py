#!/usr/bin/env python3
import http.server
import sys
import json
import os
from http import HTTPStatus

class MockHTTPHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        # Map port numbers to expected service names
        port = int(sys.argv[1])
        service_name = "unknown"
        if port == 8091:
            service_name = "mock-http-1"
        elif port == 8092:
            service_name = "mock-http-2"
        
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
        if port == 8091:
            service_name = "mock-http-1"
        elif port == 8092:
            service_name = "mock-http-2"
        
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
        sys.stderr.write(f"HTTP Server ({sys.argv[1]}): {format % args}\n")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 mock_http_server.py <port>")
        sys.exit(1)

    port = int(sys.argv[1])
    server_address = ("localhost", port)
    
    print(f"Starting HTTP server on port {port}...")
    
    try:
        # Create HTTP server with improved socket handling
        httpd = http.server.ThreadingHTTPServer(server_address, MockHTTPHandler)
        print(f"HTTP Server running on http://localhost:{port}")
        httpd.serve_forever()
    except Exception as e:
        print(f"Error starting HTTP server on port {port}: {e}")
        sys.exit(1)
