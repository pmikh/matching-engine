use crate::domain::order::{OrderId, Price, Quantity, Revision};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OrderDeletion {
    pub id: OrderId,
    pub revision: Revision,
}

#[derive(Deserialize)]
pub struct OrderModification {
    pub id: OrderId,
    pub revision: Revision,
    pub new_price: Option<Price>,
    pub new_quantity: Option<Quantity>,
}
