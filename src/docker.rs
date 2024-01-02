use bollard::{
    container::{Config, CreateContainerOptions, NetworkingConfig, StartContainerOptions},
    image::CreateImageOptions,
    Docker,
};
use futures_util::stream::StreamExt;

pub async fn pull_image(docker: &Docker, image: String) {
    println!("Starting to pull image {image}");
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });

    let mut stream = docker.create_image(options, None, None);
    while stream.next().await.is_some() {}
    println!("Image pull done.");
}

pub async fn stop_start_container(docker: &Docker, container: String) {
    println!("Container: {:?}", &container);
    let info = docker.inspect_container(&container, None).await.unwrap();

    docker.stop_container(&container, None).await.unwrap();
    docker.remove_container(&container, None).await.unwrap();

    let options = Some(CreateContainerOptions {
        name: container.clone(),
        platform: info.platform,
    });

    let container_config = info.config.unwrap();
    let network_config = NetworkingConfig {
        endpoints_config: info.network_settings.unwrap().networks.unwrap(),
    };

    let mut config = Config::from(container_config);
    config.host_config = info.host_config;
    config.networking_config = Some(network_config);

    docker
        .create_container(options, config)
        .await
        .unwrap();

    docker
        .start_container(&container, None::<StartContainerOptions<String>>)
        .await
        .unwrap();
}
