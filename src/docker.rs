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

    let config = info.config.unwrap();
    let network_config = NetworkingConfig {
        endpoints_config: info.network_settings.unwrap().networks.unwrap(),
    };

    let start_config = Config {
        hostname: config.hostname,
        domainname: config.domainname,
        user: config.user,
        attach_stdin: config.attach_stdin,
        attach_stdout: config.attach_stdout,
        attach_stderr: config.attach_stderr,
        exposed_ports: config.exposed_ports,
        tty: config.tty,
        open_stdin: config.open_stdin,
        stdin_once: config.stdin_once,
        env: config.env,
        cmd: config.cmd,
        healthcheck: config.healthcheck,
        args_escaped: None,
        image: config.image,
        volumes: config.volumes,
        working_dir: config.working_dir,
        entrypoint: config.entrypoint,
        network_disabled: config.network_disabled,
        mac_address: config.mac_address,
        on_build: config.on_build,
        labels: config.labels,
        stop_signal: config.stop_signal,
        stop_timeout: config.stop_timeout,
        shell: config.shell,
        host_config: info.host_config,
        networking_config: Some(network_config),
    };

    docker
        .create_container(options, start_config)
        .await
        .unwrap();

    docker
        .start_container(&container, None::<StartContainerOptions<String>>)
        .await
        .unwrap();
}
