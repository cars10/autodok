use core::time;

mod common;

#[tokio::test]
async fn test_it() {
    common::run_server().await;
    let docker = common::setup_docker().await.unwrap();
    let random = autodok::random::random_string(32);

    common::build_and_start_container(&docker, &random)
        .await
        .unwrap();

    let params = autodok::routes::UpdateContainerParams {
        container: "python_server".to_string(),
        wait: Some(true),
        pull: None,
    };
    let client = reqwest::Client::new();

    client
        .post("http://0.0.0.0:3000/update_container")
        .header(
            "Authorization",
            std::fs::read_to_string("data/api_key").unwrap(),
        )
        .json(&params)
        .send()
        .await
        .unwrap();

    std::thread::sleep(time::Duration::from_secs(3));
    let a = client
        .get("http://docker_server:8000/index.txt")
        .send()
        .await
        .unwrap();
    assert_eq!(random, a.text().await.unwrap().trim());
}
