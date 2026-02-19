use crate::error::AutodokError;
use bollard::{
    Docker,
    models::{ContainerCreateBody, CreateImageInfo},
    query_parameters::{CreateContainerOptions, CreateImageOptions, StartContainerOptions},
    secret::ContainerConfig,
};
use futures_util::stream::StreamExt;
use log::debug;
use std::collections::HashMap;

pub async fn pull_image(docker: &Docker, image: String) -> Result<(), AutodokError> {
    let options = Some(CreateImageOptions {
        from_image: Some(image.clone()),
        ..Default::default()
    });

    let credentials = crate::credentials::registry_credentials(&image);

    let mut stream = docker.create_image(options, None, credentials);
    while let Some(res) = stream.next().await {
        let info: CreateImageInfo = res?;
        debug!("{info:?}");
    }
    Ok(())
}

pub async fn stop_start_container(
    docker: &Docker,
    container: String,
    image: String,
) -> Result<(), crate::AutodokError> {
    let info = docker.inspect_container(&container, None).await?;

    // stop and remove old container
    docker.stop_container(&container, None).await?;
    docker.remove_container(&container, None).await?;

    // build general options for new container (platform is String in 0.20 API)
    let create_options = Some(CreateContainerOptions {
        name: Some(container.clone()),
        platform: info.platform.unwrap_or_default(),
    });

    let container_config = ContainerConfig {
        image: Some(image),
        ..info.config.unwrap()
    };

    // build network options - we can only create with a single network, the rest needs to be connected later
    let mut previous_networks = info.network_settings.unwrap().networks.unwrap();
    let default_network_name = previous_networks.keys().next().unwrap().to_string();
    let default_network = previous_networks.remove(&default_network_name).unwrap();

    let mut endpoints_config = HashMap::new();
    endpoints_config.insert(default_network_name, default_network);

    // ContainerCreateBody has the same config fields as ContainerConfig plus host_config and networking_config
    let config = ContainerCreateBody {
        hostname: container_config.hostname,
        domainname: container_config.domainname,
        user: container_config.user,
        attach_stdin: container_config.attach_stdin,
        attach_stdout: container_config.attach_stdout,
        attach_stderr: container_config.attach_stderr,
        exposed_ports: container_config.exposed_ports,
        tty: container_config.tty,
        open_stdin: container_config.open_stdin,
        stdin_once: container_config.stdin_once,
        env: container_config.env,
        cmd: container_config.cmd,
        healthcheck: container_config.healthcheck,
        args_escaped: container_config.args_escaped,
        image: container_config.image,
        volumes: container_config.volumes,
        working_dir: container_config.working_dir,
        entrypoint: container_config.entrypoint,
        network_disabled: container_config.network_disabled,
        on_build: container_config.on_build,
        labels: container_config.labels,
        stop_signal: container_config.stop_signal,
        stop_timeout: container_config.stop_timeout,
        shell: container_config.shell,
        host_config: info.host_config,
        networking_config: Some(bollard::models::NetworkingConfig {
            endpoints_config: Some(endpoints_config),
        }),
    };

    docker.create_container(create_options, config).await?;

    for (network_name, endpoint_config) in previous_networks {
        let config = bollard::secret::NetworkConnectRequest {
            container: container.clone(),
            endpoint_config: Some(endpoint_config),
        };
        docker.connect_network(&network_name, config).await.unwrap();
    }

    docker
        .start_container(&container, None::<StartContainerOptions>)
        .await?;

    Ok(())
}
