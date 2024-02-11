use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bollard::Docker;
use log::info;
use serde::Serialize;

use crate::error::AutodokError;

#[derive(Debug, Serialize)]
pub struct Msg {
    pub message: String,
}

pub async fn update_image(
    State(docker): State<Docker>,
    Path((container, image)): Path<(String, String)>,
) -> Result<Response, AutodokError> {
    docker.inspect_container(&container, None).await?;
    info!("  Container '{container}' found.");

    info!("  Pulling image '{image}'...");
    crate::docker::pull_image(&docker, image.clone()).await?;
    info!("  Image pull done.");

    info!("  Restarting container...");
    crate::docker::stop_start_container(&docker, container.clone()).await?;
    info!("  Container '{container}' restarted with new image '{image}'.");

    let msg = crate::routes::Msg {
        message: format!("Container '{container}' restarted with new image '{image}'"),
    };
    Ok((StatusCode::OK, serde_json::to_string(&msg).unwrap()).into_response())
}
