use axum::{
    Router,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
};
use axum_required_headers::Headers;
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
