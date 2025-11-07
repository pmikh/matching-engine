use crate::domain::order::{Price, Quantity};
use crate::domain::side::Side;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct OrderEntry {
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,
}

impl OrderEntry {
    pub fn new<P, Q>(price: P, quantity: Q, side: Side) -> Self
    where
        P: Into<Price>,
        Q: Into<Quantity>,
    {
        OrderEntry {
            price: price.into(),
            quantity: quantity.into(),
            side,
        }
    }
}
