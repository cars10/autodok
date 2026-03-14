use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bollard::{container::ListContainersOptions, Docker};
use serde::{Deserialize, Serialize};

use crate::docker;
use crate::error::AutodokError;
use crate::parse;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateContainerParams {
    pub container: String,
    pub pull: Option<bool>,
    pub wait_for_completion: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateContainerResponse {
    pub container: String,
    pub image: String,
}

impl UpdateContainerResponse {
    pub fn new(container: String, image: String) -> Self {
        UpdateContainerResponse { container, image }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateImageParams {
    pub previous_image: Option<String>,
    pub image: String,
    pub wait_for_completion: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateImageResponse {
    pub containers: Vec<UpdateContainerResponse>,
}

impl UpdateImageResponse {
    pub fn new(containers: Vec<UpdateContainerResponse>) -> Self {
        UpdateImageResponse { containers }
    }
}

pub async fn update_container(
    State(docker): State<Docker>,
    extract::Json(payload): extract::Json<UpdateContainerParams>,
) -> Result<Response, AutodokError> {
    let wait_for_completion = payload.wait_for_completion.unwrap_or(false);

    if wait_for_completion {
        let image = docker::pull_image_and_update_container(
            &docker,
            &payload.container,
            None,
            payload.pull,
        )
        .await?;

        let resp = UpdateContainerResponse::new(payload.container, image);
        Ok((StatusCode::OK, Json(resp)).into_response())
    } else {
        let docker = docker.clone();
        let container = payload.container.clone();
        let pull = payload.pull;

        tokio::spawn(async move {
            if let Err(err) =
                docker::pull_image_and_update_container(&docker, &container, None, pull).await
            {
                eprintln!("failed to update container {container}: {err}");
            }
        });

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

async fn perform_update_image(
    docker: Docker,
    payload: UpdateImageParams,
) -> Result<UpdateImageResponse, AutodokError> {
    let target_image = parse::parse_image_tag(payload.image)?;
    let previous_image = match payload.previous_image {
        Some(prev) => Some(parse::parse_image_tag(prev)?),
        None => None,
    };

    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    let mut updated = Vec::new();

    for container in containers {
        let container_image = match container.image {
            Some(img) => img,
            None => continue,
        };

        let matches = match &previous_image {
            Some(prev) => &container_image == prev,
            None => container_image == target_image,
        };

        if !matches {
            continue;
        }

        let container_name = container
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|name| name.trim_start_matches('/').to_string())
            .unwrap_or_else(|| container.id.unwrap_or_default());

        if container_name.is_empty() {
            continue;
        }

        let image = match &previous_image {
            Some(_) => {
                docker::pull_image_and_update_container(
                    &docker,
                    &container_name,
                    Some(target_image.clone()),
                    Some(true),
                )
                .await?
            }
            None => {
                docker::pull_image_and_update_container(
                    &docker,
                    &container_name,
                    Some(target_image.clone()),
                    Some(true),
                )
                .await?
            }
        };

        updated.push(UpdateContainerResponse::new(container_name, image));
    }

    Ok(UpdateImageResponse::new(updated))
}

pub async fn update_image(
    State(docker): State<Docker>,
    extract::Json(payload): extract::Json<UpdateImageParams>,
) -> Result<Response, AutodokError> {
    let wait_for_completion = payload.wait_for_completion.unwrap_or(false);

    if wait_for_completion {
        let resp = perform_update_image(docker.clone(), payload).await?;
        Ok((StatusCode::OK, Json(resp)).into_response())
    } else {
        tokio::spawn(async move {
            if let Err(err) = perform_update_image(docker, payload).await {
                eprintln!("failed to update image: {err}");
            }
        });

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

pub async fn health(State(docker): State<Docker>) -> Result<Response, AutodokError> {
    docker.ping().await?;
    Ok((StatusCode::OK).into_response())
}
