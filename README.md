# axum-required-headers

Ergonomic required and optional HTTP header extraction for [`axum`](https://github.com/tokio-rs/axum) applications.

## Features

- **Type-safe header extraction** with compile-time validation
- **`Required<T>`** wrapper for headers that must be present (returns 400 Bad Request if missing)
- **`Optional<T>`** wrapper for headers that may be absent (returns `None` if missing)
- **`#[derive(Headers)]`** for extracting multiple headers into a single struct
- Automatic JSON error responses with descriptive messages

## Installation

```toml
[dependencies]
axum-required-headers = "0.1"
```

## Usage

### Individual Header Types

Define a header type and use `Required<T>` or `Optional<T>` wrappers.

**Requirements for `#[derive(Header)]`:**
- Must be applied to a struct
- Requires the `#[header("header-name")]` attribute
- The type must implement `FromStr` (you provide the parsing logic)
- The `FromStr::Err` type must implement `std::error::Error`

```rust
use axum_required_headers::{Header, Required, Optional};

#[derive(Header)]
#[header("x-user-id")]
struct UserId(String);

// The type must implement `FromStr` to define how the header value is parsed
impl std::str::FromStr for UserId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

async fn handler(
    Required(user_id): Required<UserId>,      // 400 if missing
    Optional(tenant): Optional<UserId>,       // None if missing
) {
    println!("User: {}", user_id.0);
}
```

### Composite Header Structs

Extract multiple headers at once with `#[derive(Headers)]`.

**Requirements for `#[derive(Headers)]`:**
- Must be applied to a struct with named fields
- Each field requires the `#[header("header-name")]` attribute
- Field types must implement `FromStr` (e.g., `String`, `i32`, `Uuid`, or custom types)
- Fields wrapped in `Option<T>` are optional; all others are required

```rust
use axum_required_headers::Headers;

#[derive(Headers)]
pub struct AppHeaders {
    #[header("x-user-id")]
    pub user_id: String,              // Required

    #[header("x-api-version")]
    pub api_version: Option<String>,  // Optional
}

async fn handler(headers: AppHeaders) -> String {
    format!("User: {}", headers.user_id)
}
```

## Behavior Notes

- **Case insensitivity**: Header names are case-insensitive per HTTP specification. `X-User-Id`, `x-user-id`, and `X-USER-ID` are all equivalent.
- **Duplicate headers**: If a request contains multiple headers with the same name, only the **first** value is extracted.

## Error Responses

Missing or invalid headers return `400 Bad Request` with a JSON body:

```json
{
  "error": "missing_header",
  "message": "Missing required header: `x-user-id`"
}
```

Error types: `missing_header`, `invalid_header_value` (non-ASCII), `header_parse_error`

