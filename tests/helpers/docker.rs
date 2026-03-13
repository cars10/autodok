use bollard::container::{
    self, InspectContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StopContainerOptions,
};
use bollard::image::{CreateImageOptions, PushImageOptions};
use bollard::secret::{HealthConfig, HealthStatusEnum, HostConfig, PortBinding};
use bollard::{image::BuildImageOptions, Docker};
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Once;
use std::time::{Duration, Instant};

static START_SERVER: Once = Once::new();

pub async fn run_server() {
    START_SERVER.call_once(|| {
        tokio::spawn(async move {
            let config = autodok::config::Config::new();
            autodok::run(&config).await.unwrap();
        });
    });
}

pub async fn cleanup_containers(
    docker: &Docker,
    random: &str,
) -> Result<(), bollard::errors::Error> {
    let filters = {
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("AUTODOK_RANDOM_STRING={}", random)],
        );
        filters
    };

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

pub async fn setup_docker(
    random: &str,
    registry_host_port: u16,
) -> Result<Docker, bollard::errors::Error> {
    let docker = connect_docker().await;
    docker.ping().await?;

    start_registry(&docker, random, registry_host_port).await?;
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
    random: &str,
    app_host_port: u16,
    registry_host_port: u16,
) -> Result<(), bollard::errors::Error> {
    let mut build_args = HashMap::new();
    build_args.insert("MESSAGE", random);

    build_image(
        docker,
        Some(build_args),
        &format!("localhost:{registry_host_port}/python_server"),
        random,
        "tests/Dockerfile.python",
    )
    .await?;
    start_container(
        docker,
        &format!("python_server_{}", random),
        &format!("localhost:{registry_host_port}/python_server:{}", random),
        8000,
        app_host_port,
        random,
    )
    .await?;
    Ok(())
}

pub async fn build_image(
    docker: &Docker,
    build_args: Option<HashMap<&str, &str>>,
    image_name: &str,
    image_tag: &str,
    dockerfile_path: &str,
) -> Result<(), bollard::errors::Error> {
    let build_args = build_args.unwrap_or_default();

    let tar = create_dockerfile_tar(dockerfile_path).await?;

    let build_options = BuildImageOptions {
        dockerfile: dockerfile_path,
        buildargs: build_args,
        t: &format!("{image_name}:{image_tag}"),
        rm: true,
        ..Default::default()
    };

    let mut image_build_stream =
        docker.build_image(build_options, None, Some(bytes::Bytes::from(tar)));

    while let Some(res) = image_build_stream.next().await {
        res?;
    }

    let push_options = Some(PushImageOptions { tag: image_tag });
    let mut push_stream = docker.push_image(image_name, push_options, None);
    while let Some(res) = push_stream.next().await {
        res?;
    }

    Ok(())
}

async fn create_dockerfile_tar(dockerfile_path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut dockerfile = File::open(dockerfile_path)?;
    let dockerfile_size = dockerfile.metadata()?.len();

    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_path(dockerfile_path)?;
    header.set_size(dockerfile_size);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append(&header, &mut dockerfile)?;

    Ok(tar.into_inner()?)
}

async fn start_registry(
    docker: &Docker,
    random: &str,
    registry_host_port: u16,
) -> Result<(), bollard::errors::Error> {
    let pull_options = CreateImageOptions {
        from_image: "registry:2",
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(pull_options), None, None);
    while let Some(msg) = stream.next().await {
        msg?;
    }

    start_container(
        docker,
        &format!("registry_{}", random),
        "registry:2",
        5000,
        registry_host_port,
        random,
    )
    .await?;
    Ok(())
}

pub async fn start_container(
    docker: &Docker,
    container_name: &str,
    image: &str,
    container_port: u16,
    host_port: u16,
    random: &str,
) -> Result<(), bollard::errors::Error> {
    let port_bindings = create_port_bindings(container_port, host_port);

    let healthcheck = HealthConfig {
        test: Some(vec![
            "CMD-SHELL".to_string(),
            format!("wget --spider -q http://0.0.0.0:{container_port} || exit 1"),
        ]),
        start_interval: Some(0),
        interval: Some(5_000_000_000), // 5 seconds in nanoseconds
        timeout: Some(3_000_000_000),  // 3 seconds in nanoseconds
        retries: Some(3),
        start_period: Some(5_000_000_000), // 5 seconds in nanoseconds
    };

    let mut labels = HashMap::new();
    labels.insert("AUTODOK_RANDOM_STRING", random);

    let config = container::Config {
        image: Some(image),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        }),
        healthcheck: Some(healthcheck),
        labels: Some(labels),
        ..Default::default()
    };

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

fn create_port_bindings(
    container_port: u16,
    host_port: u16,
) -> HashMap<String, Option<Vec<PortBinding>>> {
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        format!("{container_port}/tcp"),
        Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(host_port.to_string()),
        }]),
    );
    port_bindings
}

pub async fn wait_for_container(
    docker: &Docker,
    container: &str,
    timeout: Option<Duration>,
) -> Result<(), bollard::errors::Error> {
    let start_time = Instant::now();

    loop {
        if start_time.elapsed() > timeout.unwrap_or(Duration::from_secs(10)) {
            return Err(bollard::errors::Error::RequestTimeoutError);
        }

        let container_info = docker
            .inspect_container(container, None::<InspectContainerOptions>)
            .await?;

        if let Some(state) = container_info.state {
            if let Some(health) = state.health {
                if let Some(HealthStatusEnum::HEALTHY) = health.status {
                    return Ok(());
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
