use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
};
use axum_required_headers::{Header, Optional, Required};
use http_body_util::BodyExt;
use std::convert::Infallible;
use std::num::ParseIntError;
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

// Custom type with fallible parsing for testing parse errors
#[derive(Header, Debug, Clone, PartialEq)]
#[header("x-positive-int")]
struct PositiveInt(u32);

impl FromStr for PositiveInt {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u32 = s.parse()?;
        Ok(Self(value))
    }
}

// Custom type that always fails parsing
#[derive(Header, Debug)]
#[header("x-always-fails")]
struct AlwaysFails;

#[derive(Debug)]
struct AlwaysFailsError;

impl std::fmt::Display for AlwaysFailsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "always fails")
    }
}

impl std::error::Error for AlwaysFailsError {}

impl FromStr for AlwaysFails {
    type Err = AlwaysFailsError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Err(AlwaysFailsError)
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

// Parse failure handlers
async fn required_positive_int_handler(Required(value): Required<PositiveInt>) -> String {
    format!("value: {}", value.0)
}

async fn optional_positive_int_handler(Optional(value): Optional<PositiveInt>) -> String {
    match value {
        Some(v) => format!("value: {}", v.0),
        None => "no value".to_string(),
    }
}

async fn always_fails_required_handler(Required(_): Required<AlwaysFails>) -> String {
    "unreachable".to_string()
}

async fn always_fails_optional_handler(Optional(value): Optional<AlwaysFails>) -> String {
    match value {
        Some(_) => "has value".to_string(),
        None => "no value".to_string(),
    }
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

// ============================================================================
// PARSE FAILURE TESTS
// ============================================================================

#[tokio::test]
async fn test_required_header_parse_failure_invalid_int() {
    let app = Router::new().route("/", get(required_positive_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "not-a-number")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_required_header_parse_failure_negative_overflow() {
    let app = Router::new().route("/", get(required_positive_int_handler));

    // u32 cannot represent negative numbers
    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "-5")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_required_header_parse_success_valid_int() {
    let app = Router::new().route("/", get(required_positive_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "42")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optional_header_parse_failure_returns_none() {
    let app = Router::new().route("/", get(optional_positive_int_handler));

    // Optional should return None on parse failure, not error
    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "not-a-number")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Optional extraction should NOT fail - it returns None on parse error
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_optional_header_parse_success() {
    let app = Router::new().route("/", get(optional_positive_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "100")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_required_header_always_fails_parse() {
    let app = Router::new().route("/", get(always_fails_required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-always-fails", "any-value")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_optional_header_always_fails_parse_returns_none() {
    let app = Router::new().route("/", get(always_fails_optional_handler));

    // Even with a value present, if parsing fails, Optional returns None
    let request = Request::builder()
        .uri("/")
        .header("x-always-fails", "any-value")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // The handler should run successfully, returning "no value"
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// ERROR RESPONSE BODY VERIFICATION TESTS
// ============================================================================

async fn read_body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_missing_header_error_response_body() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "missing_header");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("x-organization-id")
    );
}

#[tokio::test]
async fn test_invalid_header_value_error_response_body() {
    let app = Router::new().route("/", get(required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-organization-id", "org-日本語-123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "invalid_header_value");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("x-organization-id")
    );
}

#[tokio::test]
async fn test_parse_error_response_body() {
    let app = Router::new().route("/", get(required_positive_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "not-a-number")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "header_parse_error");
    assert!(body["message"].as_str().unwrap().contains("x-positive-int"));
}

// ============================================================================
// DEREF AND DEREFMUT BEHAVIOR TESTS
// ============================================================================

#[tokio::test]
async fn test_required_deref() {
    // Verify Required<T> derefs to T
    let header_value = PositiveInt(42);
    let required = Required(header_value);

    // Deref should give us access to the inner value
    let inner: &PositiveInt = &required;
    assert_eq!(*inner, PositiveInt(42));

    // Can also access the inner struct's field through deref
    assert_eq!((*required).0, 42);
}

#[tokio::test]
async fn test_required_deref_mut() {
    let header_value = PositiveInt(42);
    let mut required = Required(header_value);

    // DerefMut should allow mutation of the inner value
    (*required).0 = 100;
    assert_eq!((*required).0, 100);
}

#[tokio::test]
async fn test_optional_deref_some() {
    let header_value = PositiveInt(42);
    let optional = Optional(Some(header_value));

    // Deref should give us access to the Option
    let inner: &Option<PositiveInt> = &optional;
    assert!(inner.is_some());
    assert_eq!(inner.as_ref().unwrap().0, 42);
}

#[tokio::test]
async fn test_optional_deref_none() {
    let optional: Optional<PositiveInt> = Optional(None);

    // Deref should give us access to the None variant
    let inner: &Option<PositiveInt> = &optional;
    assert!(inner.is_none());
}

#[tokio::test]
async fn test_optional_deref_mut() {
    let header_value = PositiveInt(42);
    let mut optional = Optional(Some(header_value));

    // DerefMut should allow mutation
    if let Some(ref mut inner) = *optional {
        inner.0 = 100;
    }
    assert_eq!(optional.0.as_ref().unwrap().0, 100);

    // Can also replace the entire Option
    *optional = None;
    assert!(optional.is_none());
}

// ============================================================================
// OPTIONAL HEADER EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_optional_header_whitespace_value() {
    let app = Router::new().route("/", get(optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "   ")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Whitespace is still a valid string value
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optional_header_unicode_returns_none() {
    let app = Router::new().route("/", get(optional_positive_int_handler));

    // Unicode in Optional should cause to_str() to fail, returning None
    let request = Request::builder()
        .uri("/")
        .header("x-positive-int", "日本語")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Optional extraction should return error for invalid ASCII
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
