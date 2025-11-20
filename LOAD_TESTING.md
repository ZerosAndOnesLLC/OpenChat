# OpenChat Load Testing

## Token Cache Load Test

The `load_test_token_cache.sh` script validates that token caching is working correctly and measures its effectiveness.

### Prerequisites

- `curl` - for making HTTP requests
- `jq` - for parsing JSON responses
- `parallel` or `xargs` - for concurrent requests (parallel is recommended)
- Valid JWT token from TitaniumVault

### Installation

Install GNU parallel for better performance:

```bash
# Ubuntu/Debian
sudo apt-get install parallel

# macOS
brew install parallel

# Or use xargs (pre-installed on most systems)
```

### Usage

Basic usage with default settings (50 concurrent requests, 500 total):

```bash
TOKEN=your_jwt_token ./load_test_token_cache.sh
```

Custom configuration:

```bash
TOKEN=your_jwt_token \
API_URL=http://localhost:8080 \
CONCURRENT_REQUESTS=100 \
TOTAL_REQUESTS=1000 \
./load_test_token_cache.sh
```

### Environment Variables

- `TOKEN` (required) - JWT token from TitaniumVault
- `API_URL` (optional) - API base URL, defaults to `http://localhost:8080`
- `CONCURRENT_REQUESTS` (optional) - Number of concurrent requests, defaults to 50
- `TOTAL_REQUESTS` (optional) - Total number of requests to make, defaults to 500

### Expected Results

With proper token caching:
- **First request**: Cache miss (validates token with TitaniumVault)
- **Subsequent requests**: Cache hits for 5 minutes
- **Cache hit rate**: >99% for repeated requests with same token

Example output:

```
===== Results =====
Duration: 5s
Requests per second: 100

Cache Metrics:
{
  "total_hits": 499,
  "total_misses": 1,
  "total_operations": 500,
  "hit_rate_percentage": "99.80%",
  "by_type": {
    "tokens": {
      "hits": 499,
      "misses": 1,
      "total": 500,
      "hit_rate": "99.80%"
    }
  }
}

===== Token Cache Performance =====
Token Cache Hits: 499
Token Cache Misses: 1
Token Cache Hit Rate: 99.80%

✓ PASS: Token caching is working effectively!
```

### What This Tests

1. **Token Cache Effectiveness**: Verifies that tokens are cached and reused
2. **Concurrency**: Tests multiple simultaneous requests with the same token
3. **Performance**: Measures requests per second and response times
4. **TitaniumVault Load Reduction**: Confirms we're not hitting TV-API for every request

### Interpreting Results

#### Good Results ✓
- Hit rate > 99%
- Only 1-2 cache misses (initial requests)
- Fast response times (<100ms per request)

#### Bad Results ✗
- Hit rate < 95%
- Many cache misses
- Slow response times (>500ms per request)

If you see bad results, check:
1. Redis is running and accessible
2. Token cache TTL is set correctly (300 seconds / 5 minutes)
3. No errors in API logs
4. Redis memory is not full

### Monitoring During Test

Watch API logs in real-time:

```bash
# Local development
docker logs -f openchat-api

# Production
aws logs filter-log-events \
  --profile prod \
  --region us-east-1 \
  --log-group-name "/ecs/0n1-us-east-1/openchat-service" \
  --start-time $(($(date +%s) * 1000 - 300000)) \
  --max-items 50
```

Watch for:
- "Token cache hit" messages (should be ~99%)
- "Token cache miss" messages (should be ~1%)
- "Starting new connection to TitaniumVault" (should be minimal)

### Advanced Usage

#### Test Cache Expiration

Test that caches expire after 5 minutes:

```bash
TOKEN=your_token ./load_test_token_cache.sh

# Wait 5+ minutes
sleep 310

# Run again - first request should miss cache
TOKEN=your_token TOTAL_REQUESTS=10 ./load_test_token_cache.sh
```

#### Stress Test

Test with higher concurrency:

```bash
TOKEN=your_token \
CONCURRENT_REQUESTS=200 \
TOTAL_REQUESTS=2000 \
./load_test_token_cache.sh
```

#### Multiple Users

Test with multiple different tokens:

```bash
# Create a file with tokens (one per line)
cat > tokens.txt <<EOF
token1_here
token2_here
token3_here
EOF

# Run test for each token
while read -r TOKEN; do
  echo "Testing token..."
  TOKEN=$TOKEN TOTAL_REQUESTS=100 ./load_test_token_cache.sh
done < tokens.txt
```

### Troubleshooting

#### "API is not available"
- Ensure the API is running: `curl http://localhost:8080/health`
- Check API_URL is correct

#### "401 Unauthorized"
- Token is invalid or expired
- Get a fresh token from TitaniumVault

#### "Connection refused"
- API is not running
- Check Docker: `docker ps | grep openchat`

#### Low hit rate
- Redis might not be running: `docker ps | grep redis`
- Check API logs for cache errors
- Verify Redis connection in API configuration

## Future Load Tests

Additional load tests to add:

1. **WebSocket Connection Load Test** - Test concurrent WebSocket connections
2. **Message Throughput Test** - Test messages per second
3. **Channel Switch Test** - Test rapid channel switching
4. **Full User Simulation** - Simulate realistic user behavior
