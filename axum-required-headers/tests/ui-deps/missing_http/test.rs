//! Test that Headers derive fails when http dependency is missing

use axum_required_headers_derive::Headers;

#[derive(Headers)]
struct MyHeaders {
    #[header("x-request-id")]
    request_id: String,
}

fn main() {}
