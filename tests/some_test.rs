mod helpers;
use helpers::autodok_test::AutodokTest;

#[tokio::test]
async fn test_update_container() {
    let test = AutodokTest::new().await;
    let payload = autodok::routes::UpdateContainerParams {
        container: format!("python_server_{}", test.random),
        pull: None,
    };
    test.send("update_container", payload.clone()).await;

    helpers::docker::wait_for_container(&test.docker, &payload.container, None)
        .await
        .unwrap();

    test.check_random().await;
    test.cleanup_containers().await.unwrap();
}

#[tokio::test]
async fn test_update_image_without_previous_image() {
    let test = AutodokTest::new().await;

    let container_name = test.container_name();
    let initial_image = test.current_image();

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    let payload = serde_json::json!({
        "image": initial_image,
    });
    test.send("update_image", payload).await;

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    test.check_random().await;
    test.cleanup_containers().await.unwrap();
}

#[tokio::test]
async fn test_update_image_with_previous_image() {
    let test = AutodokTest::new().await;

    let container_name = test.container_name();
    let previous_image = test.current_image();
    let new_tag = format!("{}-new", test.random);
    let new_image = test.image_for_tag(&new_tag);

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    helpers::docker::build_image(
        &test.docker,
        None,
        &format!("localhost:{}/python_server", test.registry_host_port),
        &new_tag,
        "tests/Dockerfile.python",
    )
    .await
    .unwrap();

    let payload = serde_json::json!({
        "previous_image": previous_image,
        "image": new_image,
    });
    test.send("update_image", payload).await;

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    test.check_random().await;
    test.cleanup_containers().await.unwrap();
}
