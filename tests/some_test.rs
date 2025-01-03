use serde::Serialize;

mod helpers;

struct AutodokTest {
    docker: bollard::Docker,
    random: String,
    client: reqwest::Client
}

impl AutodokTest {
    pub async fn new() -> Self {
        helpers::docker::run_server().await;
        let random = autodok::random::random_string(32);
        let docker = helpers::docker::setup_docker().await.unwrap();
        helpers::docker::build_and_start_container(&docker, &random)
            .await
            .unwrap();

            let client = reqwest::Client::new();


        AutodokTest { docker, random, client }
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
        let response = self.client
        .get("http://docker_server:8000/index.txt")
        .send()
        .await
        .unwrap();

        assert_eq!(self.random, response.text().await.unwrap().trim());
    }
}

#[tokio::test]
async fn test_it() {
    let test = AutodokTest::new().await;
    let payload = autodok::routes::UpdateContainerParams {
        container: "python_server".to_string(),
        wait: Some(true),
        pull: None,
    };
    test.send("update_container", payload.clone()).await;

    helpers::docker::wait_for_container(&test.docker, &payload.container, None)
        .await
        .unwrap();

    test.check_random().await;
}
