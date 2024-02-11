use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use axum::{routing::get, routing::post, Router};
use bollard::errors::Error as BolladError;
use bollard::Docker;
use error::AutodokError;
use lazy_static::lazy_static;

use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tracing::{info_span, Span};

mod api_key;
mod docker;
mod error;
mod random;
mod routes;

lazy_static! {
    static ref API_KEY: String = api_key::api_key();
}

#[tokio::main]
async fn main() {
    let format = tracing_subscriber::fmt::format().with_target(false);
    tracing_subscriber::fmt().event_format(format).init();

    ctrlc::set_handler(|| {
        log::info!("Stopping autodok...");
        std::process::exit(0);
    })
    .expect("Error setting exit handler");

    log::info!("Starting autodok...");

    run().await.unwrap();
}

async fn run() -> Result<(), AutodokError> {
    let docker = connect_docker()?;
    let tracing = TraceLayer::new_for_http()
        .make_span_with(|_request: &Request<_>| {
            let req_id = random::random_string(12).to_lowercase();
            info_span!("", "r" = req_id)
        })
        .on_request(|request: &Request<_>, _span: &Span| {
            log::info!(
                "{method} {uri}",
                method = request.method(),
                uri = request.uri()
            );
        })
        .on_response(|response: &Response, latency: Duration, _span: &Span| {
            log::info!(
                "{status} in {latency}ms",
                status = response.status(),
                latency = latency.as_millis()
            );
            log::info!("");
        })
        .on_failure(
            |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                log::error!("Error: {error:?}");
            },
        );

    let app = Router::new()
        .route("/update/:container/:image", post(routes::update_image))
        .route_layer(middleware::from_fn_with_state(API_KEY.to_string(), auth))
        .route("/health", get(routes::health))
        .with_state(docker)
        .layer(tracing);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn connect_docker() -> Result<Docker, BolladError> {
    Docker::connect_with_socket_defaults()
}

async fn auth(
    State(api_key): State<String>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
