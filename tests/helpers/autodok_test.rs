use serde::Serialize;

use super::docker;

#[derive(Clone)]
pub struct AutodokTest {
    pub docker: bollard::Docker,
    pub random: String,
    client: reqwest::Client,
    pub app_host_port: u16,
    pub registry_host_port: u16,
}

impl AutodokTest {
    pub async fn new() -> Self {
        docker::run_server().await;
        let random = autodok::random::random_string(16);
        let hash = u16::from_le_bytes(random.as_bytes()[0..2].try_into().unwrap());
        let registry_host_port = 5000 + (hash % 1000); // 5000-5999
        let app_host_port = 8000 + (hash % 1000); // 8000-8999

        let docker = docker::setup_docker(&random, registry_host_port)
            .await
            .unwrap();
        docker::build_and_start_container(&docker, &random, app_host_port, registry_host_port)
            .await
            .unwrap();

        let client = reqwest::Client::new();

        AutodokTest {
            docker,
            random,
            client,
            app_host_port,
            registry_host_port,
        }
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
            .get(&format!(
                "http://docker_server:{}/index.txt",
                self.app_host_port
            ))
            .send()
            .await
            .unwrap();

        assert_eq!(self.random, response.text().await.unwrap().trim());
    }

    pub async fn cleanup_containers(&self) -> Result<(), bollard::errors::Error> {
        docker::cleanup_containers(&self.docker, &self.random).await
    }
}
