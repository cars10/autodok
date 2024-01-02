use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use bollard::Docker;

pub async fn update_image(
    State(docker): State<Docker>,
    Path((image, container)): Path<(String, String)>,
) -> Result<String, (StatusCode, String)> {
    crate::docker::pull_image(&docker, image.clone()).await;

    crate::docker::stop_start_container(&docker, container).await;

    Ok("Läuft".to_string())
}
