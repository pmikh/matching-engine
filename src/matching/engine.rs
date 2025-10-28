use crate::domain::order::Order;
use crate::domain::order_book::OrderBook;
use crate::domain::trade::Trade;
use crate::matching::command::MatchingEngineCommand;
use crate::matching::engine::MarketEvent::{OrderDeleted, OrderModified};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum MarketEvent {
    TradeExecuted(Trade),
    OrderDeleted(Order),
    OrderModified,
    OrderCreated(Order),
}

pub async fn matching_engine(
    mut rx: Receiver<MatchingEngineCommand>,
    ws_tx: broadcast::Sender<MarketEvent>,
) {
    let mut book = OrderBook::default();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            MatchingEngineCommand::Create(order_entry) => {
                if let Err(e) = ws_tx.send(MarketEvent::OrderCreated(order_entry.clone().into())) {
                    error!("Failed to broadcast message: {e}")
                };

                if let Some(trades) = book.match_order(order_entry) {
                    for trade in trades {
                        if let Err(e) = ws_tx.send(MarketEvent::TradeExecuted(trade)) {
                            error!("Failed to broadcast message: {e}")
                        };
                    }
                }
            }
            MatchingEngineCommand::Delete(id, rev) => {
                if let Ok(o) = book.delete_order(&(id, rev)) {
                    if let Err(e) = ws_tx.send(OrderDeleted(o)) {
                        error!("Failed to broadcast message: {e}")
                    };
                }
            }
            MatchingEngineCommand::Modify(id, rev, price, quantity) => {
                if let Ok(trades) = book.modify_order(id, rev, price, quantity) {
                    if let Err(e) = ws_tx.send(OrderModified) {
                        error!("Failed to broadcast message: {e}")
                    };

                    if let Some(executed_trades) = trades {
                        for trade in executed_trades {
                            if let Err(e) = ws_tx.send(MarketEvent::TradeExecuted(trade)) {
                                error!("Failed to broadcast message: {e}")
                            };
                        }
                    }
                }
            }
        }
    }
}
