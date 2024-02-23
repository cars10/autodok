use crate::error::AutodokError;
use bollard::{
    auth::DockerCredentials,
    container::{Config, CreateContainerOptions, NetworkingConfig, StartContainerOptions},
    image::CreateImageOptions,
    models::ContainerConfig,
    network::ConnectNetworkOptions,
    service::CreateImageInfo,
    Docker,
};
use futures_util::stream::StreamExt;
use log::debug;
use std::collections::HashMap;
use std::env;

pub async fn pull_image(
    docker: &Docker,
    image: String,
    credentials: Option<DockerCredentials>,
) -> Result<(), AutodokError> {
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });

    let credentials = credentials.or_else(default_credentials);

    let mut stream = docker.create_image(options, None, credentials);
    while let Some(res) = stream.next().await {
        let info: CreateImageInfo = res?;
        debug!("{info:?}");
    }
    Ok(())
}

pub fn default_credentials() -> Option<DockerCredentials> {
    Some(DockerCredentials {
        serveraddress: env::var("DEFAULT_REGISTRY_ADDRESS").ok(),
        username: env::var("DEFAULT_REGISTRY_USERNAME").ok(),
        password: env::var("DEFAULT_REGISTRY_PASSWORD").ok(),
        ..Default::default()
    })
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

    // build general options for new container
    let create_options = Some(CreateContainerOptions {
        name: container.clone(),
        platform: info.platform,
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

    let network_config = NetworkingConfig { endpoints_config };

    let mut config = Config::from(container_config);
    config.host_config = info.host_config;
    config.networking_config = Some(network_config);

    docker.create_container(create_options, config).await?;

    for (network_name, endpoint_config) in previous_networks {
        let config = ConnectNetworkOptions {
            container: &container,
            endpoint_config,
        };
        docker.connect_network(&network_name, config).await.unwrap();
    }

    docker
        .start_container(&container, None::<StartContainerOptions<String>>)
        .await?;

    Ok(())
}
