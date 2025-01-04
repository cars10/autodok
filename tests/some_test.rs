use std::collections::HashMap;

use bollard::container::{ListContainersOptions, RemoveContainerOptions, StopContainerOptions};
use serde::Serialize;

mod helpers;

#[derive(Clone)]
struct AutodokTest {
    docker: bollard::Docker,
    random: String,
    client: reqwest::Client,
}

impl AutodokTest {
    pub async fn new() -> Self {
        helpers::docker::run_server().await;
        let random = autodok::random::random_string(16);
        let docker = helpers::docker::setup_docker(&random).await.unwrap();
        helpers::docker::build_and_start_container(&docker, &random)
            .await
            .unwrap();

        let client = reqwest::Client::new();

        AutodokTest {
            docker,
            random,
            client,
        }
    }

    pub async fn send<T>(&self, path: &str, payload: T)
    where
        T: Serialize,
    {
        self.client
            .post(format!("http://0.0.0.0:3000/{}", path))
            .header(
                "Authorization",
                std::fs::read_to_string("data/api_key").unwrap(),
            )
            .json(&payload)
            .send()
            .await
            .unwrap();
    }

    pub async fn check_random(&self) {
        let response = self
            .client
            .get("http://docker_server:8000/index.txt")
            .send()
            .await
            .unwrap();

        assert_eq!(self.random, response.text().await.unwrap().trim());
    }

    pub async fn cleanup_containers(&self) -> Result<(), bollard::errors::Error> {
        let filters = {
            let mut filters = HashMap::new();
            filters.insert(
                "label".to_string(),
                vec![format!("AUTODOK_RANDOM_STRING={}", self.random)],
            );
            filters
        };

        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await?;

        for container in containers {
            if let Some(container_id) = &container.id {
                self.docker
                    .stop_container(container_id, Some(StopContainerOptions { t: 1 }))
                    .await?;

                self.docker
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
}

#[tokio::test]
async fn test_it() {
    let test = AutodokTest::new().await;
    let payload = autodok::routes::UpdateContainerParams {
        container: format!("python_server_{}", test.random),
        wait: Some(true),
        pull: None,
    };
    test.send("update_container", payload.clone()).await;

    helpers::docker::wait_for_container(&test.docker, &payload.container, None)
        .await
        .unwrap();

    test.check_random().await;
    test.cleanup_containers().await.unwrap();
}
