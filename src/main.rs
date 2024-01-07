use axum::{routing::post, Router};
use bollard::errors::Error as BolladError;
use bollard::Docker;
use env_logger::Env;
use error::AutodokError;

pub mod docker;
pub mod error;
pub mod routes;

fn connect_docker() -> Result<Docker, BolladError> {
    Docker::connect_with_socket_defaults() // TODO: make configure via ENV
}

async fn run() -> Result<(), AutodokError> {
    let docker = connect_docker()?;

    let app = Router::new()
        .route("/update/:image/:container", post(routes::update_image))
        .with_state(docker);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    ctrlc::set_handler(|| {
        log::info!("Stopping autodok...");
        std::process::exit(0);
    })
    .expect("Error setting exit handler");

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    log::info!("Starting autodok...");

    run().await.unwrap();
}
