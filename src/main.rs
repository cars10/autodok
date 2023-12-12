use axum::{routing::post, Router};
use bollard::Docker;
use dotenv::dotenv;

pub mod docker;
pub mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let docker = Docker::connect_with_local_defaults().unwrap();
    let app = Router::new()
        .route("/update/:image", post(routes::update_image))
        .with_state(docker);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
