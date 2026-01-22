use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("Missing required header: `{0}`")]
    Missing(&'static str),
    #[error("Invalid header value (not valid ASCII): `{0}`")]
    InvalidValue(&'static str),
    #[error("Failed to parse header value: `{0}`")]
    Parse(&'static str),
}

impl IntoResponse for HeaderError {
    fn into_response(self) -> Response {
        use HeaderError::*;
        let error = match self {
            Missing(_) => "missing_header",
            InvalidValue(_) => "invalid_header_value",
            Parse(_) => "header_parse_error",
        };
        let body = Json(json!({
            "error": error,
            "message": format!("{self}"),
        }));

        (StatusCode::BAD_REQUEST, body).into_response()
    }
}
