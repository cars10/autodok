mod helpers;
use helpers::autodok_test::AutodokTest;

#[tokio::test]
async fn test_update_container() {
    let test = AutodokTest::new().await;
    let container_name = test.container_name();
    let image_id_before = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();

    // simulate user building & pushing a new image for the current tag
    test.build_new_current_image("UPDATED_MESSAGE").await;

    let payload = autodok::routes::UpdateContainerParams {
        container: container_name.clone(),
        pull: Some(true),
        wait_for_completion: Some(true),
    };
    let _ = test.send("update_container", payload.clone()).await;

    helpers::docker::wait_for_container(&test.docker, &payload.container, None)
        .await
        .unwrap();

    let image_id_after = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();

    assert_ne!(image_id_before, image_id_after);
}

#[tokio::test]
async fn test_update_image_without_previous_image() {
    let test = AutodokTest::new().await;

    let container_name = test.container_name();
    let initial_image = test.current_image();
    let image_id_before = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    // simulate user building & pushing a new image for the current tag
    // (same tag, new ID)
    test.build_new_current_image("UPDATED_MESSAGE").await;

    let payload = serde_json::json!({
        "image": initial_image,
        "wait_for_completion": true,
    });
    let _ = test.send("update_image", payload).await;

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    let image_id_after = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();

    assert_ne!(image_id_before, image_id_after);
}

#[tokio::test]
async fn test_update_image_with_previous_image() {
    let test = AutodokTest::new().await;

    let container_name = test.container_name();
    let image_id_before = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();
    let previous_image = test.current_image();
    let new_tag = format!("{}-new", test.random);
    let new_image = test
        .build_new_image_with_tag(&new_tag, "DIFFERENT_MESSAGE")
        .await;

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    let payload = serde_json::json!({
        "previous_image": previous_image,
        "image": new_image,
        "wait_for_completion": true,
    });
    let _ = test.send("update_image", payload).await;

    helpers::docker::wait_for_container(&test.docker, &container_name, None)
        .await
        .unwrap();

    let image_id_after = helpers::docker::container_image_id(&test.docker, &container_name)
        .await
        .unwrap();

    assert_ne!(image_id_before, image_id_after);
}
