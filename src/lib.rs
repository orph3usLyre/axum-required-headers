//! Ergonomic header extraction for Axum.
//!
//! This crate provides a derive macro for automatically implementing
//! `FromRequestParts` for header extractors in Axum applications.
//!
//! # Examples
//!
//! ```ignore
//! use axum_headers::Headers;
//!
//! #[derive(Headers)]
//! pub struct AppHeaders {
//!     #[header("x-user-id")]
//!     pub user_id: String,
//!
//!     #[header("x-api-version", optional)]
//!     pub api_version: Option<String>,
//! }
//!
//! async fn handler(headers: AppHeaders) -> String {
//!     format!("User: {}", headers.user_id)
//! }
//! ```

pub use axum_required_headers_macro::{Header, Headers};

mod error;
mod extractors;

pub use error::HeaderError;
pub use extractors::{Optional, OptionalHeader, Required, RequiredHeader};

// Re-exports for user convenience
pub use axum;
pub use http;
