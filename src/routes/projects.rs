use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models;
use crate::schema;

pub async fn create_project(
    State(pool): State<Pool>,
    Json(new_project): Json<models::NewProject>,
) -> Result<Json<models::Project>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(schema::projects::table)
                .values(new_project)
                .returning(models::Project::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub async fn show_project(
    State(pool): State<Pool>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<models::Project>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(move |conn| {
            schema::projects::dsl::projects
                .find(&project_id)
                .select(models::Project::as_select())
                .first(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
