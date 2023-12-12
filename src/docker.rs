use bollard::{
    container::ListContainersOptions, container::RestartContainerOptions,
    image::CreateImageOptions, service::ContainerSummary, Docker,
};
use futures_util::stream::StreamExt;
use std::collections::HashMap;

pub async fn pull_image(docker: &Docker, image: String) {
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });

    let mut stream = docker.create_image(options, None, None);
    while let Some(item) = stream.next().await {
        println!("{:?}", item);
    }
}

pub async fn list_containers(docker: &Docker, image: String) -> Vec<ContainerSummary> {
    let mut filters = HashMap::new();
    filters.insert("ancestor".to_owned(), vec![image.clone()]);

    let list = ListContainersOptions {
        filters,
        ..Default::default()
    };

    docker.list_containers(Some(list)).await.unwrap()
}

pub async fn restart_container(docker: &Docker, container_id: String) {
    println!("Container ID: {:?}", &container_id);

    let options = Some(RestartContainerOptions { t: 30 });
    docker
        .restart_container(&container_id, options)
        .await
        .unwrap();
}
