//! Helper traits for custom header implementations.
//!
//! If you want to implement custom extraction logic beyond
//! what the derive macro provides, you can use these wrapper types
//! and traits to avoid orphan rule violations.

use axum::extract::FromRequestParts;
use http::request::Parts;
use std::ops::{Deref, DerefMut};

use crate::HeaderError;

/// Trait for headers that can be parsed from a string using `FromStr`.
///
/// Implement this trait to create custom header types with automatic
/// `FromRequestParts` support via the `Required<T>` wrapper.
pub trait RequiredHeader: std::str::FromStr + Send {
    const HEADER_NAME: &'static str;
}

/// Trait for optional headers that can be parsed from a string.
///
/// Implement this trait to create custom header types with automatic
/// `FromRequestParts` support via the `Optional<T>` wrapper.
pub trait OptionalHeader: std::str::FromStr + Send {
    const HEADER_NAME: &'static str;
}

/// Wrapper type for required headers implementing `RequiredHeader`.
///
/// This wrapper allows you to use `RequiredHeader` types directly in
/// Axum handlers without manual `FromRequestParts` implementation.
///
/// # Examples
///
/// ```ignore
/// use axum_headers::Required;
///
/// struct UserId(String);
///
/// impl std::str::FromStr for UserId {
///     type Err = std::convert::Infallible;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         Ok(UserId(s.to_string()))
///     }
/// }
///
/// impl RequiredHeader for UserId {
///     const HEADER_NAME: &'static str = "x-user-id";
/// }
///
/// async fn handler(Required(user_id): Required<UserId>) {
///     println!("User: {}", user_id.0);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Required<T>(pub T);

impl<T> Deref for Required<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Required<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wrapper type for optional headers implementing `OptionalHeader`.
///
/// This wrapper allows you to use `OptionalHeader` types directly in
/// Axum handlers without manual `FromRequestParts` implementation.
///
/// # Examples
///
/// ```ignore
/// use axum_headers::Optional;
///
/// struct TenantId(String);
///
/// impl std::str::FromStr for TenantId {
///     type Err = std::convert::Infallible;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         Ok(TenantId(s.to_string()))
///     }
/// }
///
/// impl OptionalHeader for TenantId {
///     const HEADER_NAME: &'static str = "x-tenant-id";
/// }
///
/// async fn handler(Optional(tenant_id): Optional<TenantId>) {
///     match tenant_id {
///         Some(id) => println!("Tenant: {}", id.0),
///         None => println!("No tenant"),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Optional<T>(pub Option<T>);

impl<T> Deref for Optional<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Optional<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Blanket implementation for `RequiredHeader` types via `Required<T>` wrapper.
impl<S, T> FromRequestParts<S> for Required<T>
where
    T: RequiredHeader + Send + Sync + Sized,
    <T as std::str::FromStr>::Err: std::error::Error + Send + 'static,
    S: Send + Sync,
{
    type Rejection = HeaderError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(T::HEADER_NAME)
            .ok_or(HeaderError::Missing(T::HEADER_NAME))?
            .to_str()
            .map_err(|_| HeaderError::InvalidValue(T::HEADER_NAME))?;

        let parsed = value
            .parse::<T>()
            .map_err(|_| HeaderError::Parse(T::HEADER_NAME))?;

        Ok(Required(parsed))
    }
}

/// Blanket implementation for `OptionalHeader` types via `Optional<T>` wrapper.
impl<S, T> FromRequestParts<S> for Optional<T>
where
    T: OptionalHeader + Sync,
    <T as std::str::FromStr>::Err: std::error::Error + Send + 'static,
    S: Send + Sync,
{
    type Rejection = HeaderError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get(T::HEADER_NAME) {
            None => Ok(Optional(None)),
            Some(header) => {
                let value = header
                    .to_str()
                    .map_err(|_| HeaderError::InvalidValue(T::HEADER_NAME))?;

                let parsed = value
                    .parse::<T>()
                    .map_err(|_| HeaderError::Parse(T::HEADER_NAME))?;

                Ok(Optional(Some(parsed)))
            }
        }
    }
}
