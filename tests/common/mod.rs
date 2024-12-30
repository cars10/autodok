use bollard::container::{
    self, ListContainersOptions, RemoveContainerOptions, StopContainerOptions,
};
use bollard::image::{CreateImageOptions, PushImageOptions};
use bollard::secret::{HostConfig, PortBinding};
use bollard::{image::BuildImageOptions, Docker};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub async fn run_server() {
    let config = autodok::config::Config::new();
    tokio::spawn(async move {
        autodok::run(&config).await.unwrap();
    });
}

pub async fn setup_docker() -> Result<Docker, bollard::errors::Error> {
    let docker = connect_docker().await;
    docker.ping().await?;

    stop_all(&docker).await?;
    start_registry(&docker).await?;
    Ok(docker)
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

pub async fn build_and_start_container(
    docker: &Docker,
    message: &str,
) -> Result<(), bollard::errors::Error> {
    build_image(&docker, message).await?;
    start_container(&docker).await?;

    Ok(())
}

pub async fn build_image(docker: &Docker, message: &str) -> Result<(), bollard::errors::Error> {
    let mut buildargs = std::collections::HashMap::new();
    buildargs.insert("MESSAGE", message);

    let dockerfile_path = "tests/Dockerfile.python";

    let mut dockerfile = File::open(dockerfile_path).unwrap();
    let dockerfile_size = dockerfile.metadata()?.len();

    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_path(dockerfile_path).unwrap();
    header.set_size(dockerfile_size);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append(&header, &mut dockerfile).unwrap();

    let build_options = BuildImageOptions {
        dockerfile: dockerfile_path,
        buildargs,
        t: "localhost:5000/python_server:latest",
        rm: true,
        ..Default::default()
    };

    let mut image_build_stream = docker.build_image(
        build_options,
        None,
        Some(bytes::Bytes::from(tar.into_inner().unwrap())),
    );

    while let Some(res) = image_build_stream.next().await {
        res?;
    }

    let push_options = Some(PushImageOptions { tag: "latest" });

    let mut push_stream = docker.push_image("localhost:5000/python_server", push_options, None);
    while let Some(res) = push_stream.next().await {
        res?;
    }

    Ok(())
}

async fn start_registry(docker: &Docker) -> Result<(), bollard::errors::Error> {
    let pull_options = CreateImageOptions {
        from_image: "registry:2",
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(pull_options), None, None);
    while let Some(msg) = stream.next().await {
        msg?;
    }

    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "5000/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("5000".to_string()),
        }]),
    );
    let config = container::Config {
        image: Some("registry:2"),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_name = "registry";
    docker
        .create_container(
            Some(container::CreateContainerOptions {
                name: container_name,
                ..Default::default()
            }),
            config,
        )
        .await?;

    docker
        .start_container(
            container_name,
            None::<container::StartContainerOptions<String>>,
        )
        .await?;

    Ok(())
}

pub async fn start_container(docker: &Docker) -> Result<(), bollard::errors::Error> {
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "8000/tcp".to_string(),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("8000".to_string()),
        }]),
    );
    let config = container::Config {
        image: Some("localhost:5000/python_server:latest"),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_name = "python_server";
    docker
        .create_container(
            Some(container::CreateContainerOptions {
                name: container_name,
                ..Default::default()
            }),
            config,
        )
        .await?;

    docker
        .start_container(
            container_name,
            None::<container::StartContainerOptions<String>>,
        )
        .await?;

    Ok(())
}

pub async fn stop_all(docker: &Docker) -> Result<(), bollard::errors::Error> {
    let filters: HashMap<String, Vec<String>> = HashMap::new();
    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        }))
        .await?;

    for container in containers {
        if let Some(container_id) = &container.id {
            docker
                .stop_container(container_id, Some(StopContainerOptions { t: 1 }))
                .await?;

            docker
                .remove_container(
                    container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;
        }
    }

    Ok(())
}
