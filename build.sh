#!/bin/bash
# Build script for Podcasterator

set -e

echo "Building for $(uname -s)..."
go build -o podcasterator
echo "Built: podcasterator"
