use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use bollard::errors::Error as BolladError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AutodokError {
    Docker(BolladError),
    DockerResponseServer {
        status_code: StatusCode,
        message: String,
    },
    Input(ImageParseError),
}

#[derive(Debug)]
pub enum ImageParseError {
    EmptyImage,
    EmptyPart(String),
}

impl Error for ImageParseError {}

impl fmt::Display for ImageParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyImage => write!(f, "image missing"),
            Self::EmptyPart(image) => write!(f, "invalid image: {image}"),
        }
    }
}

impl From<ImageParseError> for AutodokError {
    fn from(err: ImageParseError) -> Self {
        AutodokError::Input(err)
    }
}

impl AutodokError {
    fn from_bollard(code: u16, message: String) -> Self {
        let status_code = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        Self::DockerResponseServer {
            status_code,
            message,
        }
    }
}

impl Error for AutodokError {}

impl fmt::Display for AutodokError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Docker(e) => write!(f, "Docker error: {}", e),
            Self::DockerResponseServer {
                status_code,
                message,
            } => write!(f, "DockerContainerNotFound {status_code} {message}"),
            Self::Input(e) => write!(f, "InputError: {e:?}"),
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
            _ => AutodokError::Docker(err),
        }
    }
}

impl IntoResponse for AutodokError {
    fn into_response(self) -> Response {
        let (status_code, message) = match self {
            AutodokError::Docker(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("something went wrong: {e}"),
            ),
            AutodokError::DockerResponseServer {
                status_code,
                message,
            } => (status_code, message),
            AutodokError::Input(err) => {
                let message = match err {
                    ImageParseError::EmptyImage => "image missing".to_string(),
                    ImageParseError::EmptyPart(image) => format!("invalid image: {}", image),
                };
                (StatusCode::BAD_REQUEST, message)
            }
        };

        let msg = crate::routes::Msg { message };
        (status_code, serde_json::to_string(&msg).unwrap()).into_response()
    }
}
