//! Test that Headers derive fails on enums

use axum_required_headers::Headers;

#[derive(Headers)]
enum InvalidHeadersEnum {
    Variant1,
    Variant2,
}

fn main() {}
