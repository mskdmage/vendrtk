mod common;
use common::TestClient;

#[tokio::test]
async fn test_upload_file() {
    let client = TestClient::get().await;

    let response = client
        .upload_file(b"%PDF-1.4 test", "test.pdf")
        .await
        .unwrap();

    assert!(response.status().is_success());
}
