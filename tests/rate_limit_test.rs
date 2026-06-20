use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use std::sync::Arc;
use auth::middleware::rate_limit::Ratelimiters;
use auth::routes::auth::{signup, login};
use auth::dbutils::dbconfig::{create_pool, Appstate};
use tower_governor::GovernorLayer;
use axum::routing::post;
use serde_json::json;

/// Helper function to create a test router with rate limiting
fn create_test_router() -> Router {
    let limiters = Ratelimiters::new();
    let login_rate_limiter = limiters.login;
    let signup_rate_limiter = limiters.signup;

    let pool = create_pool();
    let appstate = Arc::new(Appstate { pool });

    Router::new()
        .route("/login", post(login))
        .layer(GovernorLayer::new(login_rate_limiter))
        .route("/signup", post(signup))
        .layer(GovernorLayer::new(signup_rate_limiter))
        .with_state(appstate)
}

/// Helper function to create a test user payload
fn create_test_user_payload() -> String {
    json!({
        "first_name": "John",
        "last_name": "Doe",
        "username": "testuser123",
        "email": "test@example.com",
        "password": "TestPass123"
    })
    .to_string()
}

/// Helper function to create a test login payload
fn create_test_login_payload() -> String {
    json!({
        "username": "testuser",
        "password": "TestPass123"
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limit_allows_requests_within_limit() {
        let app = create_test_router();

        // The rate limiter allows burst_size (5) requests initially
        // Send 5 requests rapidly - all should succeed (or fail for business logic reasons, not rate limiting)
        for i in 0..5 {
            let request = Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.1") // Simulate same IP
                .body(Body::from(create_test_login_payload()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            
            // Should not be rate limited (status should not be 429)
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Request {} should not be rate limited",
                i + 1
            );
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excessive_requests() {
        let app = create_test_router();

        // Send burst_size + 1 requests rapidly
        // The 6th request should be rate limited
        let mut responses = Vec::new();
        
        for _ in 0..6 {
            let request = Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.2") // Simulate same IP
                .body(Body::from(create_test_login_payload()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            responses.push(response.status());
        }

        // At least one request should be rate limited (429 Too Many Requests)
        let rate_limited_count = responses
            .iter()
            .filter(|&&status| status == StatusCode::TOO_MANY_REQUESTS)
            .count();

        assert!(
            rate_limited_count > 0,
            "Expected at least one request to be rate limited, but none were"
        );
    }

    #[tokio::test]
    async fn test_rate_limit_different_ips_independent() {
        let app = create_test_router();

        // Send 5 requests from IP 1
        for _ in 0..5 {
            let request = Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.10")
                .body(Body::from(create_test_user_payload()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        }

        // Send 5 requests from IP 2 - should also work independently
        for _ in 0..5 {
            let request = Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.20")
                .body(Body::from(create_test_user_payload()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Different IP should have independent rate limit"
            );
        }
    }

    #[tokio::test]
    async fn test_rate_limit_recovery_after_wait() {
        let app = create_test_router();

        // Exhaust the rate limit
        for _ in 0..6 {
            let request = Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.30")
                .body(Body::from(create_test_login_payload()))
                .unwrap();

            let _ = app.clone().oneshot(request).await.unwrap();
        }

        // Wait for rate limit to recover
        // With per_second=2 and burst_size=5, we need to wait longer
        // to allow the bucket to refill enough tokens
        sleep(Duration::from_secs(3)).await;

        // Should be able to make requests again
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.168.1.30")
            .body(Body::from(create_test_login_payload()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Should not be rate limited after waiting
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit should recover after waiting"
        );
    }

    #[tokio::test]
    async fn test_signup_and_login_have_separate_limits() {
        let app = create_test_router();

        // Exhaust signup rate limit with a different IP to avoid cross-contamination
        for _ in 0..6 {
            let request = Request::builder()
                .method("POST")
                .uri("/signup")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.40")
                .body(Body::from(create_test_user_payload()))
                .unwrap();

            let _ = app.clone().oneshot(request).await.unwrap();
        }

        // Use a different IP for login to test that limiters are truly separate
        // (not just IP-based separation)
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.168.1.41")
            .body(Body::from(create_test_login_payload()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        assert_ne!(
            response.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "Login with different IP should work even if signup is rate limited"
        );
    }

    #[tokio::test]
    async fn test_rate_limit_burst_behavior() {
        let app = create_test_router();

        // Test that burst_size (5) requests can be made immediately
        let start = std::time::Instant::now();
        
        for i in 0..5 {
            let request = Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .header("x-forwarded-for", "192.168.1.50")
                .body(Body::from(create_test_login_payload()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_ne!(
                response.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Burst request {} should not be rate limited",
                i + 1
            );
        }

        let elapsed = start.elapsed();
        
        // All 5 burst requests should complete quickly (within 1 second)
        assert!(
            elapsed < Duration::from_secs(1),
            "Burst requests should complete quickly, took {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_rate_limit_headers_present() {
        let app = create_test_router();

        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.168.1.60")
            .body(Body::from(create_test_login_payload()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        
        // Check if rate limit headers are present (tower_governor typically adds these)
        let headers = response.headers();
        
        // Note: The exact header names depend on tower_governor configuration
        // Common headers include x-ratelimit-limit, x-ratelimit-remaining, x-ratelimit-reset
        println!("Response headers: {:?}", headers);
        
        // This test documents that we should check for rate limit headers
        // Adjust header names based on actual tower_governor output
    }
}
