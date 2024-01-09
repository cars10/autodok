use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bollard::errors::Error as BolladError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AutodokError {
    DockerError(BolladError),
    GenericError(String),
}

impl Error for AutodokError {}

impl fmt::Display for AutodokError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DockerError(e) => write!(f, "Docker error: {}", e),
            Self::GenericError(s) => write!(f, "Generic error: {}", s),
        }
    }
}

impl From<BolladError> for AutodokError {
    fn from(err: BolladError) -> Self {
        AutodokError::DockerError(err)
    }
}

impl IntoResponse for AutodokError {
    fn into_response(self) -> Response {
        let body = match self {
            AutodokError::DockerError(e) => format!("something went wrong: {e}"),
            AutodokError::GenericError(_) => "something else went wrong".to_string(),
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
