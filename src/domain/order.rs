use crate::domain::order_entry::OrderEntry;
use crate::domain::side::Side;
use serde::{Deserialize, Serialize};
use std::iter::Sum;
use std::ops::SubAssign;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Eq, PartialOrd, PartialEq, Copy, Clone, Default, Hash)]
pub struct OrderId(pub Uuid);

impl OrderId {
    fn new() -> Self {
        let id = uuid::Uuid::new_v4();
        OrderId(id)
    }
}

impl From<Uuid> for OrderId {
    fn from(value: Uuid) -> Self {
        OrderId(value)
    }
}
#[derive(Deserialize, Serialize, Debug, Eq, PartialOrd, PartialEq, Ord, Copy, Clone)]
pub struct Price(pub i64);

impl From<i64> for Price {
    fn from(value: i64) -> Self {
        Price(value)
    }
}
#[derive(Deserialize, Serialize, Debug, Eq, PartialOrd, PartialEq, Copy, Clone, Ord)]
pub struct Quantity(pub i64);
impl From<i64> for Quantity {
    fn from(value: i64) -> Self {
        Quantity(value)
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Sum for Quantity {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Quantity(iter.map(|q| q.0).sum())
    }
}
#[derive(Deserialize, Serialize, Debug, Eq, PartialOrd, PartialEq, Hash, Copy, Clone)]
pub struct Revision(pub usize);
impl Revision {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl From<usize> for Revision {
    fn from(value: usize) -> Self {
        Revision(value)
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,
    pub revision: Revision,
}
impl Order {
    pub fn update<P, Q>(&mut self, new_price: Option<P>, new_quantity: Option<Q>)
    where
        P: Into<Price>,
        Q: Into<Quantity>,
    {
        match (new_price, new_quantity) {
            (None, None) => return,
            (price, quantity) => {
                if let Some(p) = price {
                    self.price = p.into();
                }
                if let Some(q) = quantity {
                    self.quantity = q.into();
                }
            }
        }
        self.revision.increment()
    }
}

impl From<OrderEntry> for Order {
    fn from(value: OrderEntry) -> Self {
        Order {
            id: OrderId::new(),
            price: value.price,
            quantity: value.quantity,
            side: value.side,
            revision: Revision(0),
        }
    }
}
