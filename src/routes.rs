use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bollard::Docker;

use crate::error::AutodokError;

pub async fn update_image(
    State(docker): State<Docker>,
    Path((image, container)): Path<(String, String)>,
) -> Result<Response, AutodokError> {
    docker.inspect_container(&container, None).await?;

    crate::docker::pull_image(&docker, image.clone()).await?;
    crate::docker::stop_start_container(&docker, container).await?;

    Ok((StatusCode::OK, "läuft".to_string()).into_response())
}
