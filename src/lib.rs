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

use config::Config;
use std::{path::Path, time::Duration};
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tracing::{info_span, Span};

pub mod api_key;
pub mod config;
pub mod credentials;
pub mod docker;
pub mod error;
pub mod parse;
pub mod random;
pub mod routes;

lazy_static! {
    static ref API_KEY: String = api_key::api_key();
}

pub async fn run(config: &Config) -> Result<(), AutodokError> {
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
        .route("/update_container", post(routes::update_container))
        .route("/update_image", post(routes::update_image))
        .route_layer(middleware::from_fn_with_state(API_KEY.to_string(), auth))
        .route("/health", get(routes::health))
        .with_state(docker)
        .layer(tracing);

    let addr = config.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log::info!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn connect_docker() -> Result<Docker, BolladError> {
    //Docker::connect_with_defaults()
    let cert_dir = std::env::var("DOCKER_TLS_CERTDIR").unwrap_or_else(|_| "/certs".to_string());

    // Construct paths for the certificate files
    let key_path = Path::new(&cert_dir).join("client/key.pem");
    let cert_path = Path::new(&cert_dir).join("client/cert.pem");
    let ca_path = Path::new(&cert_dir).join("client/ca.pem");

    // Connect to Docker with SSL
    Docker::connect_with_ssl(
        &std::env::var("DOCKER_HOST").unwrap(),
        &key_path,
        &cert_path,
        &ca_path,
        120,
        bollard::API_DEFAULT_VERSION,
    )
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
