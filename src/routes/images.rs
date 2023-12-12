use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use bollard::container::RestartContainerOptions;
use bollard::{container::ListContainersOptions, Docker};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImageDefinition {
    image: String,
    tag: String,
}

pub async fn update_image(
    Path(ImageDefinition { image, tag }): Path<ImageDefinition>,
) -> Result<String, (StatusCode, String)> {
    dbg!(image);
    dbg!(tag);
    Ok("Läuft".to_string())
}
