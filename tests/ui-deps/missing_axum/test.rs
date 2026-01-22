//! Test that Headers derive fails when axum dependency is missing

use axum_required_headers_macros::Headers;

#[derive(Headers)]
struct MyHeaders {
    #[header("x-request-id")]
    request_id: String,
}

fn main() {}
