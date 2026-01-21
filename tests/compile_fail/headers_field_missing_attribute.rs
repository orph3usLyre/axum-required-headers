//! Test that Headers derive fails when a field is missing #[header(...)] attribute

use axum_required_headers::Headers;

#[derive(Headers)]
struct FieldMissingAttribute {
    #[header("x-valid")]
    valid_field: String,
    
    // Missing #[header(...)] attribute
    invalid_field: String,
}

fn main() {}
