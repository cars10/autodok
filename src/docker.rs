use crate::error::AutodokError;
use bollard::{
    container::{Config, CreateContainerOptions, NetworkingConfig, StartContainerOptions},
    image::CreateImageOptions,
    models::ContainerConfig,
    service::CreateImageInfo,
    Docker,
};
use futures_util::stream::StreamExt;
use log::debug;

pub async fn pull_image(docker: &Docker, image: String) -> Result<(), AutodokError> {
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });

    let mut stream = docker.create_image(options, None, None);
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

    docker.stop_container(&container, None).await?;
    docker.remove_container(&container, None).await?;

    let create_options = Some(CreateContainerOptions {
        name: container.clone(),
        platform: info.platform,
    });

    let container_config = ContainerConfig {
        image: Some(image),
        ..info.config.unwrap()
    };
    let network_config = NetworkingConfig {
        endpoints_config: info.network_settings.unwrap().networks.unwrap(),
    };

    let mut config = Config::from(container_config);
    config.host_config = info.host_config;
    config.networking_config = Some(network_config);

    docker.create_container(create_options, config).await?;

    docker
        .start_container(&container, None::<StartContainerOptions<String>>)
        .await?;

    Ok(())
}
