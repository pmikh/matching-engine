use crate::domain::order::{OrderId, Price, Quantity, Revision};
use crate::domain::order_entry::OrderEntry;

#[derive(Clone, Debug)]
pub enum MatchingEngineCommand {
    Create(OrderEntry),
    Modify(OrderId, Revision, Option<Price>, Option<Quantity>),
    Delete(OrderId, Revision),
}
