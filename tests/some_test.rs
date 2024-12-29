mod common;

#[tokio::test]
async fn test_it() {
    let docker = common::setup_docker().await.unwrap();
    common::build_and_start_container(&docker, "baz")
        .await
        .unwrap();

    assert_eq!(1, 1);
}
