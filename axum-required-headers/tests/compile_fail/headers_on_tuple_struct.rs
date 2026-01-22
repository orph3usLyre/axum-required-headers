//! Test that Headers derive fails on tuple structs

use axum_required_headers::Headers;

#[derive(Headers)]
struct TupleStruct(String, String);

fn main() {}
