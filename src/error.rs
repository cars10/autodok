use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bollard::errors::Error as BolladError;
use serde_json::json;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AutodokError {
    DockerError(BolladError),
    DockerResponseServerError { status_code: u16, message: String },
    GenericError(String),
}

impl Error for AutodokError {}

impl fmt::Display for AutodokError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DockerError(e) => write!(f, "Docker error: {}", e),
            Self::DockerResponseServerError {
                status_code,
                message,
            } => write!(f, "DockerContainerNotFound {status_code} {message}"),
            Self::GenericError(s) => write!(f, "Generic error: {}", s),
        }
    }
}

impl From<BolladError> for AutodokError {
    fn from(err: BolladError) -> Self {
        match err {
            BolladError::DockerResponseServerError {
                status_code,
                message,
            } => AutodokError::DockerResponseServerError {
                status_code,
                message,
            },
            _ => AutodokError::GenericError("yolo".to_string()),
        }
    }
}

struct ApiResponse {
    status_code: StatusCode,
    message: String,
}

impl IntoResponse for AutodokError {
    fn into_response(self) -> Response {
        let (status_code, message) = match self {
            AutodokError::DockerError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("something went wrong: {e}"),
            ),
            AutodokError::DockerResponseServerError {
                status_code,
                message,
            } => (StatusCode::NOT_FOUND, message),
            AutodokError::GenericError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something else went wrong".to_string(),
            ),
        };

        (status_code, json!({"message": message}).to_string()).into_response()
    }
}
