mod common;

#[tokio::test]
async fn test_it() {
    let connection = common::setup_docker().await;

    assert_eq!(1, 1);
}
