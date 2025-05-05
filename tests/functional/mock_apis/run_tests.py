#!/usr/bin/env python3
import requests
import json
import sys
import time
from urllib3.exceptions import InsecureRequestWarning

# Suppress only the single warning from urllib3 needed.
requests.packages.urllib3.disable_warnings(category=InsecureRequestWarning)

# Default ports if not specified as arguments
HTTP_PORT = 8081
HTTPS_PORT = 8444

# Get port values from command line arguments if provided
if len(sys.argv) >= 3:
    HTTP_PORT = sys.argv[1]
    HTTPS_PORT = sys.argv[2]

gateway_http = f"http://localhost:{HTTP_PORT}"
gateway_https = f"https://localhost:{HTTPS_PORT}"

def test_endpoint(url, expected_service, method="GET", data=None):
    print(f"Testing {method} {url}...")
    try:
        if method == "GET":
            response = requests.get(url, verify=False, timeout=5)
        elif method == "POST":
            response = requests.post(url, data=data, verify=False, timeout=5)
        else:
            print(f"Unsupported method: {method}")
            return False
        
        response.raise_for_status()
        result = response.json()
        
        if "service" in result and result["service"] == expected_service:
            print(f"✓ {url} returned correct service: {result['service']}")
            return True
        else:
            print(f"✗ {url} returned incorrect service. Expected {expected_service}, got {result.get('service', 'unknown')}")
            return False
    except Exception as e:
        print(f"✗ Error testing {url}: {str(e)}")
        return False

def run_all_tests():
    tests = [
        {"url": f"{gateway_http}/api/http1", "expected": "mock-http-1"},
        {"url": f"{gateway_http}/api/http2", "expected": "mock-http-2"},
        {"url": f"{gateway_http}/api/https1", "expected": "mock-https-1"},
        {"url": f"{gateway_http}/api/https2", "expected": "mock-https-2"},
        {"url": f"{gateway_https}/api/http1", "expected": "mock-http-1"},
        {"url": f"{gateway_https}/api/http2", "expected": "mock-http-2"},
        {"url": f"{gateway_https}/api/https1", "expected": "mock-https-1"},
        {"url": f"{gateway_https}/api/https2", "expected": "mock-https-2"},
        {"url": f"{gateway_http}/api/https1", "expected": "mock-https-1", "method": "POST", "data": "test data"},
        {"url": f"{gateway_https}/api/https2", "expected": "mock-https-2", "method": "POST", "data": "more test data"},
    ]

    success = 0
    failures = 0

    for test in tests:
        method = test.get("method", "GET")
        data = test.get("data", None)
        result = test_endpoint(test["url"], test["expected"], method, data)
        if result:
            success += 1
        else:
            failures += 1

    print(f"\nTest Summary: {success} passed, {failures} failed")
    return failures == 0

if __name__ == "__main__":
    print("Waiting for all services to be ready...")
    time.sleep(2)
    print("Starting functional tests...")
    success = run_all_tests()
    sys.exit(0 if success else 1)
