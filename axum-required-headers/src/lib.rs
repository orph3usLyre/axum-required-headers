//! Ergonomic required/optional header extraction for `axum` projects.
//!
//! This crate provides a derive macro for automatically implementing
//! `FromRequestParts` for header extractors in Axum applications.
//!
//! # Examples
//!
//! ## Example with unit structs annotated with `Header`
//! ```
//! use axum_required_headers::{Header, Required, Optional};
//!
//! #[derive(Header)]
//! #[header("x-user-id")]
//! struct UserId(String);
//!
//! // structs annotated with the `Header` derive macro must also implement `FromStr`
//! impl std::str::FromStr for UserId {
//!     type Err = std::convert::Infallible;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         Ok(Self(s.to_owned()))
//!     }
//! }
//!
//! // Now you can use:
//! async fn handler(
//!     Required(user_id): Required<UserId>,
//!     Optional(maybe_user_id): Optional<UserId>,
//! ) { }
//! ```
//!
//! ## Example with composed structs annotated with `Headers`
//! ```
//! use axum_required_headers::Headers;
//!
//! #[derive(Headers)]
//! pub struct AppHeaders {
//!     #[header("x-user-id")]
//!     pub user_id: String,
//!
//!     #[header("x-api-version")]
//!     pub api_version: Option<String>,
//! }
//!
//! async fn handler(headers: AppHeaders) -> String {
//!     format!("User: {}", headers.user_id)
//! }
//!
//! async fn other_handler(headers: AppHeaders) -> String {
//!     if let Some(v) = headers.api_version {
//!         format!("Api version: {v}")
//!     } else {
//!         "No api version".to_string()
//!     }
//! }
//! ```

mod error;
mod extractors;

pub use axum_required_headers_macros::{Header, Headers};
pub use error::HeaderError;
pub use extractors::{Optional, OptionalHeader, Required, RequiredHeader};

// Re-exports for convenience
pub use axum;
pub use http;
