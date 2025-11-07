use exchange::configuration::get_configuration;
use exchange::matching::engine::{MarketEvent, matching_engine};
use exchange::matching::state::AppState;
use exchange::startup::run;
use std::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration!");
    let listener = TcpListener::bind(configuration.address())?;

    let (tx, rx) = mpsc::channel(configuration.application.matching_buffer);

    let (ws_tx, _) = broadcast::channel::<MarketEvent>(1000);

    tokio::spawn(matching_engine(rx, ws_tx.clone()));

    let state = AppState { tx, ws_tx };
    run(listener, state)?.await
}
