use axum::http::StatusCode;
use crate::common;

#[tokio::test]
async fn test_rate_limit_returns_429_when_exceeded() {
    // Allow only 2 requests per 60s window
    let (app, _pool, _guard) = common::test_app_rate_limited(2).await;

    // First two requests should succeed
    let (s1, _) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(s1, StatusCode::OK, "First request should pass");

    let (s2, _) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(s2, StatusCode::OK, "Second request should pass");

    // Third request should be rate limited
    let (s3, body) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(s3, StatusCode::TOO_MANY_REQUESTS, "Third request should be rate limited");
    assert_eq!(body["kind"], "RateLimited");
}

#[tokio::test]
async fn test_rate_limit_separate_keys() {
    // Allow only 1 request per key
    let (app, _pool, _guard) = common::test_app_rate_limited(1).await;

    // district9 first request passes
    let (s1, _) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(s1, StatusCode::OK);

    // district12 first request should also pass (different key)
    let (s2, _) = common::get_with_court(&app, "/api/attorneys", "district12").await;
    assert_eq!(s2, StatusCode::OK);

    // district9 second request should be rate limited
    let (s3, _) = common::get_with_court(&app, "/api/attorneys", "district9").await;
    assert_eq!(s3, StatusCode::TOO_MANY_REQUESTS);
}
