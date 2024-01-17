use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bollard::errors::Error as BolladError;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AutodokError {
    DockerError(BolladError),
    DockerResponseServerError {
        status_code: StatusCode,
        message: String,
    },
    GenericError(String),
}

impl AutodokError {
    fn from_bollard(code: u16, message: String) -> Self {
        let status_code = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Self::DockerResponseServerError {
            status_code,
            message,
        }
    }
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
            } => AutodokError::from_bollard(status_code, message),
            _ => AutodokError::DockerError(err),
        }
    }
}

#[derive(Debug, Serialize)]
struct Msg {
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
            } => (status_code, message),
            AutodokError::GenericError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something else went wrong".to_string(),
            ),
        };

        let msg = Msg { message };
        (status_code, serde_json::to_string(&msg).unwrap()).into_response()
    }
}
