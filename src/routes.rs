use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bollard::Docker;
use serde::{Deserialize, Serialize};

use crate::docker;
use crate::error::AutodokError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateContainerParams {
    pub container: String,
    pub wait: Option<bool>,
    pub pull: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateContainerResponse {
    pub container: String,
    pub image: String,
    pub wait: bool,
}

impl UpdateContainerResponse {
    pub fn new(container: String, image: String, wait: bool) -> Self {
        UpdateContainerResponse {
            container,
            image,
            wait,
        }
    }
}

pub async fn update_container(
    State(docker): State<Docker>,
    extract::Json(payload): extract::Json<UpdateContainerParams>,
) -> Result<Response, AutodokError> {
    let image = docker::pull_image_and_update_container(
        &docker,
        &payload.container,
        None,
        payload.wait,
        payload.pull,
    )
    .await?;

    let resp =
        UpdateContainerResponse::new(payload.container, image, payload.wait.unwrap_or(false));
    Ok((StatusCode::OK, Json(resp)).into_response())
}

pub async fn health(State(docker): State<Docker>) -> Result<Response, AutodokError> {
    docker.ping().await?;
    Ok((StatusCode::OK).into_response())
}
