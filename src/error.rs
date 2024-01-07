use bollard::errors::Error as BolladError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AutodokError {
    DockerError(BolladError),
}

impl Error for AutodokError {}

impl fmt::Display for AutodokError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DockerError(e) => write!(f, "Docker error: {}", e),
        }
    }
}

impl From<BolladError> for AutodokError {
    fn from(err: BolladError) -> Self {
        AutodokError::DockerError(err)
    }
}
