# Rate Limiting Tests Documentation

This document explains the rate limiting tests for the authentication service and how to use them effectively.

## Overview

The rate limiting implementation uses `tower_governor` with IP-based rate limiting. The configuration is:
- **Rate**: 2 requests per second
- **Burst size**: 5 requests
- **Key extractor**: SmartIpKeyExtractor (IP-based)

## Test Suite

### Running the Tests

```bash
# Run all rate limit tests
cargo test --test rate_limit_test

# Run a specific test
cargo test --test rate_limit_test test_rate_limit_blocks_excessive_requests

# Run with output
cargo test --test rate_limit_test -- --nocapture
```

### Test Cases

#### 1. `test_rate_limit_allows_requests_within_limit`
**Purpose**: Verifies that requests within the burst limit are allowed.

**What it tests**:
- Sends 5 rapid requests (the burst size)
- All should succeed without rate limiting (status ≠ 429)

**Expected behavior**: All 5 requests pass through without being rate limited.

---

#### 2. `test_rate_limit_blocks_excessive_requests`
**Purpose**: Verifies that excessive requests are blocked.

**What it tests**:
- Sends 6 rapid requests (burst size + 1)
- At least one should be rate limited (status = 429)

**Expected behavior**: The 6th request (or later) receives a 429 Too Many Requests response.

---

#### 3. `test_rate_limit_different_ips_independent`
**Purpose**: Verifies that different IPs have independent rate limits.

**What it tests**:
- Sends 5 requests from IP 192.168.1.10
- Sends 5 requests from IP 192.168.1.20
- Both sets should succeed independently

**Expected behavior**: Rate limits are tracked per IP, not globally.

---

#### 4. `test_rate_limit_recovery_after_wait`
**Purpose**: Verifies that rate limits recover over time.

**What it tests**:
- Exhausts the rate limit (6 requests)
- Waits 3 seconds
- Makes another request

**Expected behavior**: After waiting, the rate limit recovers and new requests are allowed.

**Note**: The wait time (3 seconds) is calculated based on:
- Rate: 2 requests/second
- Burst exhausted: 5 tokens used
- Recovery time: ~2.5 seconds to refill enough tokens

---

#### 5. `test_signup_and_login_have_separate_limits`
**Purpose**: Verifies that signup and login endpoints have separate rate limiters.

**What it tests**:
- Exhausts signup rate limit from one IP
- Makes login request from different IP
- Login should work independently

**Expected behavior**: Each endpoint has its own rate limiter instance.

---

#### 6. `test_rate_limit_burst_behavior`
**Purpose**: Verifies that burst requests complete quickly.

**What it tests**:
- Sends 5 burst requests
- Measures time taken
- Should complete in under 1 second

**Expected behavior**: Burst capacity allows rapid requests without delay.

---

#### 7. `test_rate_limit_headers_present`
**Purpose**: Documents that rate limit headers should be checked.

**What it tests**:
- Makes a request
- Prints response headers for inspection

**Expected behavior**: Rate limit headers (if configured) should be present in responses.

**Note**: This is a documentation test. Adjust based on actual header names from tower_governor.

---

## Understanding Rate Limiting Behavior

### Token Bucket Algorithm

The rate limiter uses a token bucket algorithm:

1. **Burst Size (5)**: Initial tokens available
2. **Refill Rate (2/sec)**: Tokens added per second
3. **Request Cost (1)**: Tokens consumed per request

### Example Scenarios

#### Scenario 1: Rapid Burst
```
Time 0s: 5 tokens available
- Request 1: ✓ (4 tokens left)
- Request 2: ✓ (3 tokens left)
- Request 3: ✓ (2 tokens left)
- Request 4: ✓ (1 token left)
- Request 5: ✓ (0 tokens left)
- Request 6: ✗ (429 Too Many Requests)
```

#### Scenario 2: Sustained Rate
```
Time 0s: 5 tokens, make 2 requests (3 left)
Time 1s: 5 tokens (refilled 2), make 2 requests (3 left)
Time 2s: 5 tokens (refilled 2), make 2 requests (3 left)
...continues indefinitely at 2 req/sec
```

#### Scenario 3: Recovery
```
Time 0s: Exhaust all 5 tokens
Time 1s: 2 tokens available (refilled)
Time 2s: 4 tokens available (refilled)
Time 3s: 5 tokens available (capped at burst size)
```

## Customizing Tests

### Testing Different Rate Limits

To test with different rate limits, modify the `Ratelimiters::new()` configuration:

```rust
let login = Arc::new(
    GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(5)      // Change rate
        .burst_size(10)     // Change burst
        .finish()
        .unwrap()
);
```

### Testing Different IPs

Use the `x-forwarded-for` header to simulate different IPs:

```rust
.header("x-forwarded-for", "192.168.1.100")
```

### Testing Different Endpoints

Add routes to the test router:

```rust
Router::new()
    .route("/custom", post(custom_handler))
    .layer(GovernorLayer::new(custom_rate_limiter))
```

## Troubleshooting

### Test Failures

**Problem**: `test_rate_limit_recovery_after_wait` fails
- **Cause**: Wait time too short
- **Solution**: Increase sleep duration in the test

**Problem**: `test_rate_limit_blocks_excessive_requests` fails
- **Cause**: Burst size allows more requests than expected
- **Solution**: Send more requests (burst_size + 2 or more)

**Problem**: Tests are flaky
- **Cause**: Timing issues or shared state
- **Solution**: Use unique IPs per test, increase wait times

### Common Issues

1. **Rate limiter state persists**: Each test creates a new router, so state is isolated
2. **IP extraction fails**: Ensure `x-forwarded-for` header is set correctly
3. **Database connection issues**: Rate limit tests don't require database, but handlers might

## Best Practices

1. **Use unique IPs per test**: Prevents cross-test interference
2. **Wait for recovery**: Add sufficient sleep time for token refill
3. **Test edge cases**: Burst exhaustion, recovery, concurrent requests
4. **Monitor headers**: Check rate limit headers in responses
5. **Test realistic scenarios**: Simulate actual user behavior patterns

## Integration with CI/CD

Add to your CI pipeline:

```yaml
- name: Run rate limit tests
  run: cargo test --test rate_limit_test
```

Consider:
- Running tests in parallel (they use different IPs)
- Setting timeouts for long-running tests
- Collecting test metrics for monitoring

## Further Reading

- [tower_governor documentation](https://docs.rs/tower-governor/)
- [governor crate](https://docs.rs/governor/)
- [Token bucket algorithm](https://en.wikipedia.org/wiki/Token_bucket)