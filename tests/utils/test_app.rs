use exchange::matching::engine::matching_engine;
use exchange::matching::state::AppState;
use exchange::startup;
use std::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

pub struct TestApp {
    pub address: String,
}

pub fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to the random port");
    let port = listener.local_addr().unwrap().port();

    let address = format!("http://127.0.0.1:{}", port);

    let (tx, rx) = mpsc::channel(10_000);
    let (ws_tx, _) = broadcast::channel(1000);

    tokio::spawn(matching_engine(rx, ws_tx.clone()));

    let state = AppState { tx, ws_tx };
    let server = startup::run(listener, state).expect("Test server was not created successfully");

    tokio::spawn(server);

    TestApp { address }
}
