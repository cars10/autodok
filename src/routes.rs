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

#[derive(Debug, Deserialize)]
pub struct UpdateContainerImage {
    container: String,
    image: String,
}

#[derive(Debug, Serialize)]
pub struct UpdatedResponse {
    pub container: String,
    pub image: String,
}

impl UpdatedResponse {
    pub fn new(container: String, image: String) -> Self {
        UpdatedResponse { container, image }
    }
}

pub async fn update_container(
    State(docker): State<Docker>,
    extract::Json(payload): extract::Json<UpdateContainerImage>,
) -> Result<Response, AutodokError> {
    let image = crate::parse::parse_image_tag(payload.image.to_string())?;

    docker::update_image_and_container(&docker, &payload.container, &image).await?;
    let resp = UpdatedResponse::new(payload.container, payload.image);
    Ok((StatusCode::OK, Json(resp)).into_response())
}

pub async fn health(State(docker): State<Docker>) -> Result<Response, AutodokError> {
    docker.ping().await?;
    Ok((StatusCode::OK).into_response())
}
