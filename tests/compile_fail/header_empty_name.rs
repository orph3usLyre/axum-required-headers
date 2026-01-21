//! Test that Header derive fails with empty header name

use axum_required_headers::Header;
use std::str::FromStr;

#[derive(Header)]
#[header("")]
struct EmptyHeaderName(String);

impl FromStr for EmptyHeaderName {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

fn main() {}
