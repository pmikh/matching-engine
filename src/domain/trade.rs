use crate::domain::order::{OrderId, Price, Quantity};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Trade {
    pub price: Price,
    pub quantity: Quantity,
    maker_id: OrderId,
    taker_id: OrderId,
    exec_time: i64,
}

pub fn now_unix_ns() -> i64 {
    let since_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time flows backwards?!");

    since_unix.as_nanos() as i64
}
impl Trade {
    pub fn new<P, Q, I>(price: P, quantity: Q, maker_id: I, taker_id: I) -> Self
    where
        P: Into<Price>,
        Q: Into<Quantity>,
        I: Into<OrderId>,
    {
        let exec_time = now_unix_ns();

        Trade {
            price: price.into(),
            quantity: quantity.into(),
            maker_id: maker_id.into(),
            taker_id: taker_id.into(),
            exec_time,
        }
    }
}
