#!/bin/bash
# Entry point script for running functional tests

set -e

# Check for required dependencies
command -v openssl >/dev/null 2>&1 || { echo "Error: openssl is required but not installed. Aborting."; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "Error: python3 is required but not installed. Aborting."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo is required but not installed. Aborting."; exit 1; }

# Check if python requests module is installed and install if needed, without prompting
python3 -c "import requests" >/dev/null 2>&1 || { 
  echo "Installing Python 'requests' module..."
  pip3 install requests >/dev/null 2>&1
  if [ $? -ne 0 ]; then
    echo "Failed to install 'requests' module. Please install it manually with: pip3 install requests"
    exit 1
  else
    echo "Successfully installed Python 'requests' module."
  fi
}

# Set the working directory to the location of this script
cd "$(dirname "$0")"

# Run the makefile with the specified target or 'all' by default
if [ -z "$1" ]; then
  make all
else
  make "$@"
fi
