use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
};
use axum_required_headers::{Header, Optional, Required};
use std::convert::Infallible;
use std::str::FromStr;
use tower::ServiceExt;

#[derive(Header)]
#[header("x-organization-id")]
struct OrganizationId(String);

impl FromStr for OrganizationId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

#[derive(Header)]
#[header("x-user-id")]
struct UserId(String);

impl FromStr for UserId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

// Test handlers

async fn required_handler(Required(org_id): Required<OrganizationId>) -> String {
    format!("org: {}", org_id.0)
}

async fn optional_handler(Optional(user_id): Optional<UserId>) -> String {
    match user_id {
        Some(id) => format!("user: {}", id.0),
        None => "no user".to_string(),
    }
}

async fn both_handler(
    Required(org_id): Required<OrganizationId>,
    Optional(user_id): Optional<UserId>,
) -> String {
    let user_part = user_id.map(|id| id.0).unwrap_or_else(|| "none".to_string());
    format!("org: {}, user: {}", org_id.0, user_part)
}

async fn multiple_same_type_handler(
    Required(org_id): Required<OrganizationId>,
    Optional(other_org): Optional<OrganizationId>,
) -> String {
    let other_part = other_org
        .map(|id| id.0)
        .unwrap_or_else(|| "none".to_string());
    format!("org: {}, other: {}", org_id.0, other_part)
}

// ============================================================================
// REQUIRED HEADER TESTS
// ============================================================================

#[tokio::test]
async fn test_required_header_present() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_required_header_missing() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_required_header_empty_value() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_required_header_special_characters() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123!@#$%^&*()")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_required_header_whitespace() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "  org-123  ")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// OPTIONAL HEADER TESTS
// ============================================================================

#[tokio::test]
async fn test_optional_header_present() {
    let app = Router::new().route("/", get(optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user-456")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optional_header_missing() {
    let app = Router::new().route("/", get(optional_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optional_header_empty_value() {
    let app = Router::new().route("/", get(optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// MIXED REQUIRED AND OPTIONAL TESTS
// ============================================================================

#[tokio::test]
async fn test_both_required_present_optional_missing() {
    let app = Router::new().route("/", get(both_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_both_required_present_optional_present() {
    let app = Router::new().route("/", get(both_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .header("x-user-id", "user-456")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_both_required_missing_optional_present() {
    let app = Router::new().route("/", get(both_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user-456")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_both_required_missing_optional_missing() {
    let app = Router::new().route("/", get(both_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// MULTIPLE SAME TYPE TESTS
// ============================================================================

#[tokio::test]
async fn test_same_type_required_present_optional_missing() {
    let app = Router::new().route("/", get(multiple_same_type_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_same_type_required_present_optional_present() {
    let app = Router::new().route("/", get(multiple_same_type_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_same_type_required_missing() {
    let app = Router::new().route("/", get(multiple_same_type_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// CASE SENSITIVITY TESTS
// ============================================================================

#[tokio::test]
async fn test_header_name_case_sensitivity_lowercase() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_header_name_case_insensitive_uppercase() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("X-ORGANIZATION-ID", "org-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // HTTP headers are case-insensitive, so this should work
    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// UNICODE AND ENCODING TESTS
// ============================================================================

#[tokio::test]
// Only accept ascii
async fn test_required_header_unicode_value() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-日本語-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_required_header_long_value() {
    let app = Router::new().route("/", get(required_handler));

    let long_value = "org-".to_string() + &"x".repeat(1000);

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", &long_value)
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// MULTIPLE HANDLER CALLS TESTS
// ============================================================================

#[tokio::test]
async fn test_sequential_requests_with_different_headers() {
    let app = Router::new().route("/", get(required_handler));

    // First request
    let request1 = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-1")
        .body(axum::body::Body::empty())
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();

    assert_eq!(response1.status(), StatusCode::OK);

    // Second request with different value
    let request2 = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-2")
        .body(axum::body::Body::empty())
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();

    assert_eq!(response2.status(), StatusCode::OK);
}
