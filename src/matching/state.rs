use crate::matching::command::MatchingEngineCommand;
use crate::matching::engine::MarketEvent;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct AppState {
    pub tx: Sender<MatchingEngineCommand>,
    pub ws_tx: broadcast::Sender<MarketEvent>,
}
