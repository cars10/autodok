use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use bollard::Docker;

pub async fn update_image(
    State(docker): State<Docker>,
    Path(image): Path<String>,
) -> Result<String, (StatusCode, String)> {
    crate::docker::pull_image(&docker, image.clone()).await;

    let containers = crate::docker::list_containers(&docker, image).await;
    for container in containers {
        let container_id = container.id.unwrap().clone();
        crate::docker::restart_container(&docker, container_id).await;
    }

    Ok("Läuft".to_string())
}
