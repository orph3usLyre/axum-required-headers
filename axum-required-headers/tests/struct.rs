use axum::{
    Router,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use axum_required_headers::Headers;
use http_body_util::BodyExt;
use std::num::ParseIntError;
use std::str::FromStr;
use tower::util::ServiceExt;

#[derive(Headers)]
pub struct TestHeaders {
    #[header("x-user-id")]
    pub user_id: String,

    #[header("x-optional")]
    pub optional_field: Option<String>,
}

async fn test_handler(headers: TestHeaders) -> impl IntoResponse {
    let s = format!("user: {}", headers.user_id);
    println!("{s}");
    s
}

// ============================================================================
// BASIC HEADERS DERIVE TESTS
// ============================================================================

#[tokio::test]
async fn test_required_headers_present() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_required_headers_missing() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_optional_headers_missing() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_optional_headers_present() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user123")
        .header("x-optional", "some-value")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// PARSE FAILURE TESTS FOR HEADERS DERIVE
// ============================================================================

// Custom parseable type for testing parse failures
#[derive(Debug, Clone)]
pub struct PositiveInt(u32);

impl FromStr for PositiveInt {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u32 = s.parse()?;
        Ok(Self(value))
    }
}

#[derive(Headers)]
pub struct ParseableHeaders {
    #[header("x-count")]
    pub count: PositiveInt,

    #[header("x-optional-count")]
    pub optional_count: Option<PositiveInt>,
}

async fn parseable_handler(headers: ParseableHeaders) -> impl IntoResponse {
    let opt_str = headers
        .optional_count
        .map(|c| c.0.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!("count: {}, optional: {}", headers.count.0, opt_str)
}

#[tokio::test]
async fn test_headers_required_field_parse_success() {
    let app = Router::new().route("/", get(parseable_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-count", "42")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_headers_required_field_parse_failure() {
    let app = Router::new().route("/", get(parseable_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-count", "not-a-number")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_headers_optional_field_parse_failure_returns_none() {
    let app = Router::new().route("/", get(parseable_handler));

    // Required field valid, optional field invalid - optional should return None
    let request = Request::builder()
        .uri("/")
        .header("x-count", "42")
        .header("x-optional-count", "not-a-number")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should succeed - optional parse failures become None
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_headers_optional_field_parse_success() {
    let app = Router::new().route("/", get(parseable_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-count", "42")
        .header("x-optional-count", "100")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// MULTIPLE FIELDS TESTS
// ============================================================================

#[derive(Headers)]
pub struct MultipleRequiredHeaders {
    #[header("x-first")]
    pub first: String,

    #[header("x-second")]
    pub second: String,

    #[header("x-third")]
    pub third: String,
}

async fn multiple_required_handler(headers: MultipleRequiredHeaders) -> impl IntoResponse {
    format!("{}, {}, {}", headers.first, headers.second, headers.third)
}

#[tokio::test]
async fn test_multiple_required_all_present() {
    let app = Router::new().route("/", get(multiple_required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-first", "one")
        .header("x-second", "two")
        .header("x-third", "three")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_required_first_missing() {
    let app = Router::new().route("/", get(multiple_required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-second", "two")
        .header("x-third", "three")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_multiple_required_middle_missing() {
    let app = Router::new().route("/", get(multiple_required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-first", "one")
        .header("x-third", "three")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_multiple_required_last_missing() {
    let app = Router::new().route("/", get(multiple_required_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-first", "one")
        .header("x-second", "two")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[derive(Headers)]
pub struct MultipleOptionalHeaders {
    #[header("x-opt-first")]
    pub first: Option<String>,

    #[header("x-opt-second")]
    pub second: Option<String>,

    #[header("x-opt-third")]
    pub third: Option<String>,
}

async fn multiple_optional_handler(headers: MultipleOptionalHeaders) -> impl IntoResponse {
    let first = headers.first.unwrap_or_else(|| "none".to_string());
    let second = headers.second.unwrap_or_else(|| "none".to_string());
    let third = headers.third.unwrap_or_else(|| "none".to_string());
    format!("{}, {}, {}", first, second, third)
}

#[tokio::test]
async fn test_multiple_optional_all_missing() {
    let app = Router::new().route("/", get(multiple_optional_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_optional_some_present() {
    let app = Router::new().route("/", get(multiple_optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-opt-first", "one")
        .header("x-opt-third", "three")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// ERROR RESPONSE BODY VERIFICATION FOR HEADERS DERIVE
// ============================================================================

async fn read_body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_headers_missing_error_body() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "missing_header");
    assert!(body["message"].as_str().unwrap().contains("x-user-id"));
}

#[tokio::test]
async fn test_headers_parse_error_body() {
    let app = Router::new().route("/", get(parseable_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-count", "invalid")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "header_parse_error");
    assert!(body["message"].as_str().unwrap().contains("x-count"));
}

#[tokio::test]
async fn test_headers_invalid_ascii_error_body() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "日本語")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_body_json(response).await;
    assert_eq!(body["error"], "invalid_header_value");
    assert!(body["message"].as_str().unwrap().contains("x-user-id"));
}

// ============================================================================
// WHITESPACE AND EDGE VALUE TESTS
// ============================================================================

#[tokio::test]
async fn test_headers_empty_string_value() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Empty string is a valid value
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_headers_whitespace_only_value() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "   ")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Whitespace-only is a valid string value
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_headers_optional_whitespace_only() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-user-id", "user123")
        .header("x-optional", "   ")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// GENERIC STRUCT TESTS
// ============================================================================

#[derive(Headers)]
pub struct GenericHeaders<T: FromStr + Send>
where
    T::Err: std::error::Error,
{
    #[header("x-value")]
    pub value: T,
}

async fn generic_string_handler(headers: GenericHeaders<String>) -> impl IntoResponse {
    format!("value: {}", headers.value)
}

async fn generic_int_handler(headers: GenericHeaders<i32>) -> impl IntoResponse {
    format!("value: {}", headers.value)
}

#[tokio::test]
async fn test_generic_headers_with_string() {
    let app = Router::new().route("/", get(generic_string_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-value", "hello-world")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_generic_headers_with_int_success() {
    let app = Router::new().route("/", get(generic_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-value", "42")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_generic_headers_with_int_parse_failure() {
    let app = Router::new().route("/", get(generic_int_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-value", "not-an-int")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[derive(Headers)]
pub struct GenericOptionalHeaders<T: FromStr + Send>
where
    T::Err: std::error::Error,
{
    #[header("x-required")]
    pub required: String,

    #[header("x-optional-generic")]
    pub optional_generic: Option<T>,
}

async fn generic_optional_handler(headers: GenericOptionalHeaders<i32>) -> impl IntoResponse {
    let opt_str = headers
        .optional_generic
        .map(|v| v.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!("required: {}, optional: {}", headers.required, opt_str)
}

#[tokio::test]
async fn test_generic_optional_present() {
    let app = Router::new().route("/", get(generic_optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-required", "hello")
        .header("x-optional-generic", "123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_generic_optional_missing() {
    let app = Router::new().route("/", get(generic_optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-required", "hello")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_generic_optional_parse_failure_returns_none() {
    let app = Router::new().route("/", get(generic_optional_handler));

    let request = Request::builder()
        .uri("/")
        .header("x-required", "hello")
        .header("x-optional-generic", "not-an-int")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should succeed - parse failure on optional becomes None
    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// CASE INSENSITIVITY FOR HEADERS DERIVE
// ============================================================================

#[tokio::test]
async fn test_headers_case_insensitive() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("X-USER-ID", "user123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_headers_mixed_case() {
    let app = Router::new().route("/", get(test_handler));

    let request = Request::builder()
        .uri("/")
        .header("X-User-Id", "user123")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
