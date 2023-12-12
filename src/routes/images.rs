use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use bollard::container::RestartContainerOptions;
use bollard::{container::ListContainersOptions, Docker};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models;
use crate::schema;

pub async fn create_image(
    State(pool): State<Pool>,
    Json(new_image): Json<models::NewImage>,
) -> Result<Json<models::Image>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(schema::images::table)
                .values(new_image)
                .returning(models::Image::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub async fn show_image(
    State(pool): State<Pool>,
    Path(image_id): Path<Uuid>,
) -> Result<Json<models::Image>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(move |conn| {
            schema::images::dsl::images
                .find(&image_id)
                .select(models::Image::as_select())
                .first(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    println!("{}", res.image);

    let docker = Docker::connect_with_local_defaults().unwrap();

    // Example: List all containers
    let mut filters = HashMap::new();
    filters.insert("ancestor".to_owned(), vec![res.image.clone()]);

    let list = ListContainersOptions {
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(list)).await.unwrap();
    for container in containers {
        println!("Container ID: {:?}", &container.id);

        let options = Some(RestartContainerOptions { t: 30 });
        let id = container.id.clone().unwrap();
        docker.restart_container(&id, options).await.unwrap();
    }

    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
