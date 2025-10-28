use crate::domain::order::{Price, Quantity};

#[derive(Debug, PartialOrd, PartialEq)]
pub struct OrderBookLevel {
    pub(crate) price: Price,
    pub(crate) quantity: Quantity,
}

impl OrderBookLevel {
    pub fn new<P, Q>(price: P, quantity: Q) -> Self
    where
        P: Into<Price>,
        Q: Into<Quantity>,
    {
        OrderBookLevel {
            price: price.into(),
            quantity: quantity.into(),
        }
    }
}
