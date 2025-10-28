use exchange::domain::order::Price;
use exchange::domain::order_entry::OrderEntry;
use exchange::domain::side::Side;
use exchange::matching::command::MatchingEngineCommand;
use exchange::matching::engine::{MarketEvent, matching_engine};

#[tokio::test]
async fn test_matching_engine_broadcasts_trade() {
    use tokio::sync::{broadcast, mpsc};

    let (cmd_tx, cmd_rx) = mpsc::channel(10);
    let (event_tx, _) = broadcast::channel(10);

    let engine_tx = event_tx.clone();
    tokio::spawn(matching_engine(cmd_rx, engine_tx));

    let mut event_rx = event_tx.subscribe();

    let buy_order = OrderEntry::new(100, 10, Side::Buy);

    cmd_tx
        .send(MatchingEngineCommand::Create(buy_order.clone()))
        .await
        .unwrap();

    let sell_order = OrderEntry::new(100, 10, Side::Sell);
    cmd_tx
        .send(MatchingEngineCommand::Create(sell_order.clone()))
        .await
        .unwrap();

    let event1 = event_rx.recv().await.unwrap();
    match event1 {
        MarketEvent::OrderCreated(order) => {
            assert_eq!(order.price, buy_order.price);
            assert_eq!(order.quantity, sell_order.quantity);
        }
        _ => assert!(
            false,
            "Expected MarketEvent::TradeExecuted, got: {:?}",
            event1
        ),
    }

    let event2 = event_rx.recv().await.unwrap();
    match event2 {
        MarketEvent::OrderCreated(order) => {
            assert_eq!(order.price, buy_order.price);
            assert_eq!(order.quantity, sell_order.quantity);
        }
        _ => assert!(
            false,
            "Expected MarketEvent::TradeExecuted, got: {:?}",
            event2
        ),
    }

    let event3 = event_rx.recv().await.unwrap();
    match event3 {
        MarketEvent::TradeExecuted(trade) => {
            assert_eq!(trade.price, buy_order.price);
            assert_eq!(trade.quantity, sell_order.quantity);
        }
        _ => assert!(
            false,
            "Expected MarketEvent::TradeExecuted, got: {:?}",
            event3
        ),
    }
}

#[tokio::test]
async fn test_matching_engine_broadcasts_trade_after_modification() {
    use tokio::sync::{broadcast, mpsc};

    let (cmd_tx, cmd_rx) = mpsc::channel(10);
    let (event_tx, _) = broadcast::channel(10);

    let engine_tx = event_tx.clone();
    tokio::spawn(matching_engine(cmd_rx, engine_tx));

    let mut event_rx = event_tx.subscribe();

    let buy_order = OrderEntry::new(100, 10, Side::Buy);

    cmd_tx
        .send(MatchingEngineCommand::Create(buy_order.clone()))
        .await
        .unwrap();

    let sell_order = OrderEntry::new(120, 10, Side::Sell);
    cmd_tx
        .send(MatchingEngineCommand::Create(sell_order.clone()))
        .await
        .unwrap();

    // skip first 2 events
    let _ = event_rx.recv().await.unwrap();
    let second_order = event_rx.recv().await.unwrap();

    match second_order {
        MarketEvent::OrderCreated(order) => cmd_tx
            .send(MatchingEngineCommand::Modify(
                order.id,
                order.revision,
                Some(Price(100)),
                None,
            ))
            .await
            .unwrap(),
        _ => assert!(
            false,
            "Expected MarketEvent::OrderCreated, got: {:?}",
            second_order
        ),
    }

    let modification_event = event_rx.recv().await.unwrap();
    match modification_event {
        MarketEvent::OrderModified => {}
        _ => assert!(false, "Expected MarketEvent::OrderModified",),
    }

    let trade_event = event_rx.recv().await.unwrap();

    match trade_event {
        MarketEvent::TradeExecuted(trade) => {
            assert_eq!(trade.price, buy_order.price);
            assert_eq!(trade.quantity, buy_order.quantity);
        }
        _ => assert!(
            false,
            "Expected MarketEvent::TradeExecuted, got: {:?}",
            trade_event
        ),
    }
}
