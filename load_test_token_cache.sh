#!/bin/bash

# Load test script for token caching
# This script simulates multiple concurrent requests to verify token cache performance

set -e

API_URL="${API_URL:-http://localhost:9876}"
TOKEN="${TOKEN:-}"
CONCURRENT_REQUESTS="${CONCURRENT_REQUESTS:-50}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-500}"

if [ -z "$TOKEN" ]; then
    echo "Error: TOKEN environment variable must be set"
    echo "Usage: TOKEN=your_jwt_token ./load_test_token_cache.sh"
    exit 1
fi

echo "===== OpenChat Token Cache Load Test ====="
echo "API URL: $API_URL"
echo "Concurrent Requests: $CONCURRENT_REQUESTS"
echo "Total Requests: $TOTAL_REQUESTS"
echo ""

# Check if metrics endpoint is available
echo "Checking API availability..."
if ! curl -s -f "$API_URL/health" > /dev/null 2>&1; then
    echo "Error: API is not available at $API_URL"
    exit 1
fi
echo "API is available ✓"
echo ""

# Reset metrics before test
echo "Resetting cache metrics..."
curl -s -X POST "$API_URL/api/metrics/cache/reset" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
echo "Metrics reset ✓"
echo ""

# Function to make a single request
make_request() {
    curl -s -w "%{http_code}" -o /dev/null \
      "$API_URL/api/channels" \
      -H "Authorization: Bearer $TOKEN"
}

export -f make_request
export API_URL TOKEN

echo "Starting load test..."
echo "$(date '+%Y-%m-%d %H:%M:%S') - Test started"

START_TIME=$(date +%s)

# Use GNU parallel if available, otherwise use xargs
if command -v parallel &> /dev/null; then
    echo "Using GNU parallel for concurrent requests..."
    seq 1 $TOTAL_REQUESTS | parallel -j $CONCURRENT_REQUESTS make_request > /dev/null
elif command -v xargs &> /dev/null; then
    echo "Using xargs for concurrent requests..."
    seq 1 $TOTAL_REQUESTS | xargs -I {} -P $CONCURRENT_REQUESTS bash -c 'make_request' > /dev/null
else
    echo "Warning: Neither parallel nor xargs found. Running sequentially..."
    for i in $(seq 1 $TOTAL_REQUESTS); do
        make_request > /dev/null
    done
fi

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "$(date '+%Y-%m-%d %H:%M:%S') - Test completed"
echo ""

# Get metrics
echo "Fetching cache metrics..."
METRICS=$(curl -s "$API_URL/api/metrics/cache" -H "Authorization: Bearer $TOKEN")

echo "===== Results ====="
echo "Duration: ${DURATION}s"
echo "Requests per second: $((TOTAL_REQUESTS / DURATION))"
echo ""
echo "Cache Metrics:"
echo "$METRICS" | jq '.'
echo ""

# Extract token-specific metrics
TOKEN_HITS=$(echo "$METRICS" | jq -r '.by_type.tokens.hits // 0')
TOKEN_MISSES=$(echo "$METRICS" | jq -r '.by_type.tokens.misses // 0')
TOKEN_HIT_RATE=$(echo "$METRICS" | jq -r '.by_type.tokens.hit_rate // "0.00%"')

echo "===== Token Cache Performance ====="
echo "Token Cache Hits: $TOKEN_HITS"
echo "Token Cache Misses: $TOKEN_MISSES"
echo "Token Cache Hit Rate: $TOKEN_HIT_RATE"
echo ""

# Calculate expected results
EXPECTED_MISSES=1  # First request should miss
EXPECTED_HITS=$((TOTAL_REQUESTS - EXPECTED_MISSES))

if [ "$TOKEN_HITS" -ge "$((EXPECTED_HITS - 10))" ]; then
    echo "✓ PASS: Token caching is working effectively!"
    echo "  Expected ~$EXPECTED_HITS hits, got $TOKEN_HITS"
else
    echo "✗ FAIL: Token cache hit rate is lower than expected"
    echo "  Expected ~$EXPECTED_HITS hits, got $TOKEN_HITS"
    exit 1
fi
