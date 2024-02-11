use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use axum::{routing::post, Router};
use bollard::errors::Error as BolladError;
use bollard::Docker;
use env_logger::Env;
use error::AutodokError;
use lazy_static::lazy_static;

mod api_key;
pub mod docker;
pub mod error;
pub mod routes;

lazy_static! {
    static ref API_KEY: String = api_key::api_key();
}

fn connect_docker() -> Result<Docker, BolladError> {
    Docker::connect_with_socket_defaults()
}

async fn auth(State(api_key): State<String>, req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let Some(auth_header) = auth_header else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if auth_header.eq(&api_key) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn run() -> Result<(), AutodokError> {
    let docker = connect_docker()?;

    let app = Router::new()
        .route("/update/:image/:container", post(routes::update_image))
        .route_layer(middleware::from_fn_with_state(API_KEY.to_string(), auth))
        .with_state(docker);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

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
