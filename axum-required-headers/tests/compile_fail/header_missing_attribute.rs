//! Test that Header derive fails without #[header(...)] attribute

use axum_required_headers::Header;
use std::str::FromStr;

#[derive(Header)]
struct MissingHeaderAttribute(String);

impl FromStr for MissingHeaderAttribute {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

fn main() {}
