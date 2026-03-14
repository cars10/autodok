use serde::Serialize;
use std::time::Duration;

use super::docker;

#[derive(Clone)]
pub struct AutodokTest {
    pub docker: bollard::Docker,
    pub random: String,
    client: reqwest::Client,
    pub registry_host_port: u16,
    pub api_host_port: u16,
}

impl AutodokTest {
    pub async fn new() -> Self {
        let random = autodok::random::random_string(16);
        let hash = u16::from_le_bytes(random.as_bytes()[0..2].try_into().unwrap());
        let registry_host_port = 5000 + (hash % 1000); // 5000-5999
        let app_host_port = 8000 + (hash % 1000); // 8000-8999
        let api_host_port = 9000 + (hash % 1000); // 9000-9999

        docker::run_server(api_host_port).await;

        let docker = docker::setup_docker(&random, registry_host_port)
            .await
            .unwrap();
        docker::build_and_start_container(&docker, &random, app_host_port, registry_host_port)
            .await
            .unwrap();

        let client = reqwest::Client::new();

        let test = AutodokTest {
            docker,
            random,
            client,
            registry_host_port,
            api_host_port,
        };

        test.wait_for_api().await;

        test
    }

    pub fn container_name(&self) -> String {
        format!("python_server_{}", self.random)
    }

    pub fn image_for_tag(&self, tag: &str) -> String {
        format!(
            "localhost:{}/python_server:{}",
            self.registry_host_port, tag
        )
    }

    pub fn current_image(&self) -> String {
        self.image_for_tag(&self.random)
    }

    pub async fn send<T>(&self, path: &str, payload: T) -> reqwest::Response
    where
        T: Serialize,
    {
        self.client
            .post(format!("http://0.0.0.0:{}/{}", self.api_host_port, path))
            .header(
                "Authorization",
                std::fs::read_to_string("data/api_key").unwrap(),
            )
            .json(&payload)
            .send()
            .await
            .unwrap()
    }

    async fn wait_for_api(&self) {
        let url = format!("http://0.0.0.0:{}/health", self.api_host_port);
        let api_key = std::fs::read_to_string("data/api_key").unwrap();

        for _ in 0..100 {
            let resp = self
                .client
                .get(&url)
                .header("Authorization", &api_key)
                .send()
                .await;

            if resp.is_ok() {
                return;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        panic!("API server on {} did not become ready in time", url);
    }

    pub async fn build_new_current_image(&self, message: &str) {
        let mut build_args = std::collections::HashMap::new();
        build_args.insert("MESSAGE", message);

        docker::build_image(
            &self.docker,
            Some(build_args),
            &format!("localhost:{}/python_server", self.registry_host_port),
            &self.random,
            "tests/Dockerfile.python",
        )
        .await
        .unwrap();
    }

    pub async fn build_new_image_with_tag(&self, tag: &str, message: &str) -> String {
        let mut build_args = std::collections::HashMap::new();
        build_args.insert("MESSAGE", message);

        docker::build_image(
            &self.docker,
            Some(build_args),
            &format!("localhost:{}/python_server", self.registry_host_port),
            tag,
            "tests/Dockerfile.python",
        )
        .await
        .unwrap();

        self.image_for_tag(tag)
    }
}
