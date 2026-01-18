use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            Self::BadRequest(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::Internal(ref message) => (StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
            Self::Validation(ref message) => (StatusCode::UNPROCESSABLE_ENTITY, message.clone()),
            Self::Conflict(ref message) => (StatusCode::CONFLICT, message.clone()),
            Self::Unauthorized(ref message) => (StatusCode::UNAUTHORIZED, message.clone()),
        };

        tracing::error!("API error: {}", error_message);

        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}

// Utility methods for common errors
impl ApiError {
    pub fn not_found(resource: &str, id: impl std::fmt::Display) -> Self {
        Self::NotFound(format!("{} with ID {} not found", resource, id))
    }

    pub fn not_found_simple(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    
    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(_errors: validator::ValidationErrors) -> Self {
        todo!("Convert validation errors to ApiError::Validation");
    }
}