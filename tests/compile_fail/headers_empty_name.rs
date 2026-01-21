//! Test that Headers derive fails with empty header name on field

use axum_required_headers::Headers;

#[derive(Headers)]
struct EmptyFieldHeaderName {
    #[header("")]
    invalid_field: String,
}

fn main() {}
