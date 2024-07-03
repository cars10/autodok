use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bollard::Docker;
use log::info;
use serde::{Deserialize, Serialize};

use crate::error::AutodokError;

#[derive(Debug, Deserialize)]
pub struct UpdateContainerImage {
    container: String,
    image: String,
}

#[derive(Debug, Serialize)]
pub struct Msg {
    pub message: String,
}

pub async fn update_image(
    State(docker): State<Docker>,
    extract::Json(payload): extract::Json<UpdateContainerImage>,
) -> Result<Response, AutodokError> {
    let container = payload.container;
    let image = crate::parse::parse_image_tag(payload.image)?;

    docker.inspect_container(&container, None).await?;
    info!("  Container '{container}' found.");

    info!("  Pulling image '{image}'...");
    crate::docker::pull_image(&docker, image.clone()).await?;
    info!("  Image pull done.");

    info!("  Restarting container...");
    crate::docker::stop_start_container(&docker, container.clone(), image.clone()).await?;
    info!("  Container '{container}' restarted with new image '{image}'.");

    let msg = Msg {
        message: format!("Container '{container}' restarted with new image '{image}'"),
    };
    Ok((StatusCode::OK, serde_json::to_string(&msg).unwrap()).into_response())
}

pub async fn health(State(docker): State<Docker>) -> Result<Response, AutodokError> {
    docker.info().await?;
    Ok((StatusCode::OK).into_response())
}
