use crate::utils::test_app::spawn_app;

mod utils;

#[tokio::test]
async fn health_check_returns_200() {
    let app = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to get the response!");

    assert!(response.status().is_success());
}
