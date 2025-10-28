use crate::domain::order::{Order, OrderId, Price, Quantity, Revision};
use crate::domain::order_book_level::OrderBookLevel;
use crate::domain::side::Side;
use crate::domain::trade::Trade;
use slotmap::{SlotMap, new_key_type};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::ops::Sub;

new_key_type! { pub struct OrderKey; }

#[derive(Default, Debug)]
pub struct OrderBook {
    bid: BTreeMap<Price, VecDeque<OrderKey>>,
    ask: BTreeMap<Price, VecDeque<OrderKey>>,
    indexed: HashMap<(OrderId, Revision), OrderKey>,
    orders: SlotMap<OrderKey, Order>,
}

impl Sub for Quantity {
    type Output = Quantity;

    fn sub(self, rhs: Self) -> Self::Output {
        Quantity(self.0 - rhs.0)
    }
}

#[derive(Debug)]
pub enum OrderModificationError {
    OrderNotFound,
}

impl OrderBook {
    pub fn best_of_book(&self) -> (Option<OrderBookLevel>, Option<OrderBookLevel>) {
        let best_bid = self
            .bid
            .last_key_value()
            .map(|(&price, order_keys)| OrderBookLevel {
                price,
                quantity: order_keys
                    .iter()
                    .filter_map(|k| self.orders.get(*k))
                    .map(|o| o.quantity)
                    .sum(),
            });
        let best_ask = self
            .ask
            .first_key_value()
            .map(|(&price, order_keys)| OrderBookLevel {
                price,
                quantity: order_keys
                    .iter()
                    .filter_map(|k| self.orders.get(*k))
                    .map(|o| o.quantity)
                    .sum(),
            });

        (best_bid, best_ask)
    }

    fn add_to_book<O: Into<Order>>(&mut self, order_entry: O) -> OrderId {
        let order = order_entry.into();
        let Order {
            id,
            revision,
            side,
            price,
            ..
        } = order;
        let key = self.orders.insert(order);
        self.indexed.insert((id, revision), key);

        match side {
            Side::Buy => {
                self.bid.entry(price).or_default().push_back(key);
            }
            Side::Sell => {
                self.ask.entry(price).or_default().push_back(key);
            }
        }
        id
    }

    pub fn delete_order(
        &mut self,
        key: &(OrderId, Revision),
    ) -> Result<Order, OrderModificationError> {
        if let Some(order_key) = self.indexed.remove(key) {
            self.indexed.remove(key);
            let removed = self.orders.remove(order_key).unwrap();
            Ok(removed)
        } else {
            Err(OrderModificationError::OrderNotFound)
        }
    }

    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        revision: Revision,
        price: Option<Price>,
        quantity: Option<Quantity>,
    ) -> Result<Option<Vec<Trade>>, OrderModificationError> {
        let key = (order_id, revision);
        if let Ok(mut order) = self.delete_order(&key) {
            order.update(price, quantity);

            let trades = self.match_order(order);
            Ok(trades)
        } else {
            Err(OrderModificationError::OrderNotFound)
        }
    }
    pub fn match_order<O: Into<Order>>(&mut self, order_entry: O) -> Option<Vec<Trade>> {
        let mut new_order = order_entry.into();
        let mut remaining_quantity = new_order.quantity;
        let mut trades = Vec::with_capacity(8);
        let mut prices_to_remove = Vec::with_capacity(4);

        #[inline(always)]
        fn matching_loop(
            order_keys: &mut VecDeque<OrderKey>,
            orders: &mut SlotMap<OrderKey, Order>,
            indexed: &mut HashMap<(OrderId, Revision), OrderKey>,
            mut remaining_quantity: Quantity,
            trades: &mut Vec<Trade>,
            taker_id: OrderId,
        ) -> Quantity {
            let zero_quantity = Quantity(0);
            while remaining_quantity > zero_quantity && !order_keys.is_empty() {
                let key = order_keys.front().unwrap();
                let order = orders.get_mut(*key).expect("Order must exist");

                let trade_quantity = remaining_quantity.min(order.quantity);

                trades.push(Trade::new(order.price, trade_quantity, order.id, taker_id));

                remaining_quantity -= trade_quantity;
                let old_index = (order.id, order.revision);
                order.update(None::<Price>, Some(order.quantity - trade_quantity));

                let new_index = (order.id, order.revision);

                if let Some(order_key) = indexed.remove(&old_index) {
                    indexed.insert(new_index, order_key);
                }
                if order.quantity == zero_quantity {
                    order_keys.pop_front();
                }
            }

            remaining_quantity
        }

        match new_order.side {
            Side::Buy => {
                let matching_side = &mut self.ask;

                for (&price, order_keys) in matching_side {
                    if new_order.price < price {
                        break;
                    }

                    remaining_quantity = matching_loop(
                        order_keys,
                        &mut self.orders,
                        &mut self.indexed,
                        remaining_quantity,
                        &mut trades,
                        new_order.id,
                    );

                    if order_keys.is_empty() {
                        prices_to_remove.push(price);
                    }

                    if remaining_quantity == Quantity(0) {
                        break;
                    }
                }

                for price in prices_to_remove {
                    self.ask.remove(&price);
                }
            }
            Side::Sell => {
                let matching_side = &mut self.bid;
                for (&price, order_keys) in matching_side.iter_mut().rev() {
                    if new_order.price > price {
                        break;
                    }

                    remaining_quantity = matching_loop(
                        order_keys,
                        &mut self.orders,
                        &mut self.indexed,
                        remaining_quantity,
                        &mut trades,
                        new_order.id,
                    );

                    if order_keys.is_empty() {
                        prices_to_remove.push(price);
                    }

                    if remaining_quantity == Quantity(0) {
                        break;
                    }
                }

                for price in prices_to_remove {
                    self.bid.remove(&price);
                }
            }
        }

        new_order.quantity = remaining_quantity;
        if remaining_quantity > Quantity(0) {
            self.add_to_book(new_order);
        }

        if trades.is_empty() {
            None
        } else {
            Some(trades)
        }
    }

    fn get_order<I, R>(&self, order_id: I, revision: R) -> Option<&Order>
    where
        I: Into<OrderId>,
        R: Into<Revision>,
    {
        let k = (order_id.into(), revision.into());
        self.indexed.get(&k).and_then(|o_k| self.orders.get(*o_k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order_entry::OrderEntry;
    use std::vec;
    use uuid::Uuid;

    #[test]
    fn top_of_book_returns_best_level() {
        let inputs = vec![
            (vec![], (None, None)),
            (
                vec![OrderEntry::new(15, 6, Side::Buy)],
                (Some(OrderBookLevel::new(15, 6)), None),
            ),
            (
                vec![
                    OrderEntry::new(15, 6, Side::Buy),
                    OrderEntry::new(20, 6, Side::Sell),
                ],
                (
                    Some(OrderBookLevel::new(15, 6)),
                    Some(OrderBookLevel::new(20, 6)),
                ),
            ),
            (
                vec![
                    OrderEntry::new(15, 6, Side::Buy),
                    OrderEntry::new(17, 6, Side::Buy),
                    OrderEntry::new(20, 6, Side::Sell),
                    OrderEntry::new(18, 4, Side::Sell),
                ],
                (
                    Some(OrderBookLevel::new(17, 6)),
                    Some(OrderBookLevel::new(18, 4)),
                ),
            ),
        ];

        for (orders, (real_best_bid, real_best_ask)) in inputs {
            let mut book = OrderBook::default();

            for o in orders {
                book.add_to_book(o);
            }

            let (best_bid, best_ask) = book.best_of_book();

            assert_eq!(
                best_bid, real_best_bid,
                "failed Best of Book for {real_best_bid:?} {real_best_ask:?}"
            );
        }
    }

    #[test]
    fn buy_order_full_matching() {
        let test_cases = vec![
            (
                vec![
                    OrderEntry::new(20, 6, Side::Sell),
                    OrderEntry::new(18, 4, Side::Sell),
                ],
                OrderEntry::new(21, 4, Side::Buy),
                vec![Trade::new(18, 4, Uuid::new_v4(), Uuid::new_v4())],
                1,
            ),
            (
                vec![
                    OrderEntry::new(20, 6, Side::Sell),
                    OrderEntry::new(18, 4, Side::Sell),
                ],
                OrderEntry::new(21, 5, Side::Buy),
                vec![
                    Trade::new(18, 4, Uuid::new_v4(), Uuid::new_v4()),
                    Trade::new(20, 1, Uuid::new_v4(), Uuid::new_v4()),
                ],
                1,
            ),
            (
                vec![
                    OrderEntry::new(18, 4, Side::Sell),
                    OrderEntry::new(18, 6, Side::Sell),
                ],
                OrderEntry::new(21, 5, Side::Buy),
                vec![
                    Trade::new(18, 4, Uuid::new_v4(), Uuid::new_v4()),
                    Trade::new(18, 1, Uuid::new_v4(), Uuid::new_v4()),
                ],
                1,
            ),
        ];

        for (initial_orders, incoming_order, expected_trades, orders_left) in test_cases {
            let mut book = OrderBook::default();

            for order in initial_orders {
                book.add_to_book(order);
            }

            let trades = book
                .match_order(incoming_order)
                .expect("Expected some trades");

            assert_eq!(
                trades.len(),
                expected_trades.len(),
                "Number of trades does not match"
            );

            for (trade, expected) in trades.iter().zip(expected_trades.iter()) {
                assert_eq!(trade.price, expected.price, "Trade price mismatch");
            }

            assert_eq!(book.ask.len(), orders_left, "Orders are missing")
        }
    }

    #[test]
    fn buy_order_no_matching_stays_in_book() {
        let mut book = OrderBook::default();

        let incoming_order = OrderEntry::new(10, 10, Side::Buy);

        let trades = book.match_order(incoming_order);

        assert!(trades.is_none());

        assert_eq!(book.bid.len(), 1)
    }

    #[test]
    fn sell_order_full_matching() {
        let test_cases = vec![
            (
                vec![
                    OrderEntry::new(18, 4, Side::Buy),
                    OrderEntry::new(20, 6, Side::Buy),
                ],
                OrderEntry::new(17, 4, Side::Sell),
                vec![Trade::new(20, 4, Uuid::new_v4(), Uuid::new_v4())],
                2, // remaining buy price levels
            ),
            (
                vec![
                    OrderEntry::new(18, 4, Side::Buy),
                    OrderEntry::new(20, 6, Side::Buy),
                ],
                OrderEntry::new(19, 5, Side::Sell),
                vec![Trade::new(20, 5, Uuid::new_v4(), Uuid::new_v4())],
                2,
            ),
            (
                vec![
                    OrderEntry::new(18, 4, Side::Buy),
                    OrderEntry::new(18, 6, Side::Buy),
                ],
                OrderEntry::new(17, 10, Side::Sell),
                vec![
                    Trade::new(18, 4, Uuid::new_v4(), Uuid::new_v4()),
                    Trade::new(18, 6, Uuid::new_v4(), Uuid::new_v4()),
                ],
                0,
            ),
        ];

        for (initial_orders, incoming_order, expected_trades, orders_left) in test_cases {
            let mut book = OrderBook::default();

            for order in initial_orders {
                book.add_to_book(order);
            }

            let trades = book
                .match_order(incoming_order)
                .expect("Expected some trades");

            assert_eq!(
                trades.len(),
                expected_trades.len(),
                "Number of trades does not match"
            );

            for (trade, expected) in trades.iter().zip(expected_trades.iter()) {
                assert_eq!(trade.price, expected.price, "Trade price mismatch");
            }

            assert_eq!(book.bid.len(), orders_left, "Orders are missing");
        }
    }

    #[test]
    fn sell_order_no_matching_stays_in_book() {
        let mut book = OrderBook::default();

        let incoming_order = OrderEntry::new(30, 10, Side::Sell);

        let trades = book.match_order(incoming_order);

        assert!(trades.is_none());

        assert_eq!(book.ask.len(), 1, "Sell order should remain in book");
    }

    #[test]
    fn match_results_in_maker_revision_update() {
        let mut book = OrderBook::default();

        let maker_order = OrderEntry::new(100, 5, Side::Sell);
        let o_id = book.add_to_book(maker_order);

        let taker_order = OrderEntry::new(100, 4, Side::Buy);

        book.match_order(taker_order);

        book.get_order(o_id, 1)
            .expect("Order has not updated revision!");
    }
}
