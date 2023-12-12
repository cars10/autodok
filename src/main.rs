use axum::{routing::post, Router};
use dotenv::dotenv;

pub mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new().route("/update/:image/:tag", post(routes::images::update_image));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
