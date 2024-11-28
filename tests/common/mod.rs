use bollard::Docker;
use std::path::Path;

pub async fn setup_docker() -> Docker {
    let docker = connect_docker().await;
    docker.ping().await.unwrap();
    docker
}

async fn connect_docker() -> Docker {
    Docker::connect_with_ssl(
        &std::env::var("DOCKER_HOST").unwrap(),
        Path::new("/certs/client/key.pem"),
        Path::new("/certs/client/cert.pem"),
        Path::new("/certs/client/ca.pem"),
        120,
        bollard::API_DEFAULT_VERSION,
    )
    .unwrap()
}
