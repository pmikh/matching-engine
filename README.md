## Exchange matching engine in rust

A minimal matching engine written in Rust.
It exposes simple REST endpoints for order management and broadcasts real-time market events over WebSockets.

### Overview

This project implements a naive order matching engine intended for experimentation and performance evaluation.

The matching engine runs asynchronously in its own task for efficiency.

Exposes three REST API endpoints for creating, modifying, and deleting orders.

Publishes real-time market events (trades, order updates, deletions) over WebSockets.

All numeric values are stored as integers for speed. It is assumed that any integer divided by 100 represents a real
float value.

Example:

price = 250 → 2.50

quantity = 1750 → 17.50

### Architecture

```css
┌────────────────────────────┐
│ REST API

(
Axum

)
│
│

/
orders

(
POST /PATCH/ DEL

)
│
└────────────┬───────────────┘
│
▼
┌─────────────────┐
│ Matching Engine │
│

(
runs async

)
│
└─────────────────┘
│
▼
┌─────────────────┐
│ Broadcast Layer │──► WebSocket clients
│

(
Trades, Events

)
│
└─────────────────┘

```

### Configuration

Configuration is loaded via the Settings structure from exchange::configuration::get_configuration.

Example config.yaml:

```yaml
application:
  host: 127.0.0.1
  port: 8000
  matching_buffer: 100000
  log_level: info
```

Default configuration:

```
Host: 127.0.0.1

Port: 8000

Matching buffer: 100,000
```

### API Endpoints

| Method | Endpoint | Description              |
|--------|----------|--------------------------|
| POST   | /orders  | Create a new order       |
| PATCH  | /orders  | Modify an existing order |
| DELETE | /orders  | Cancel an existing order |

All endpoints accept and return JSON.

Example: Create order

```
curl -X POST http://127.0.0.1:8000/orders \
-H "Content-Type: application/json" \
-d '[{"price": 250, "quantity": 1000, "side": "Buy"}]'
```

### WebSocket Events

Connect to /ws to receive live updates on trades and order book changes.

Event types:

* TradeExecuted

* OrderCreated

* OrderModified

* OrderDeleted

Example message:

```{
"TradeExecuted": {
"price": 250,
"quantity": 1000,
"maker_id": "00000000-0000-0000-0000-000000000000",
"taker_id": "00000000-0000-0000-0000-000000000000",
"exec_time": 1761679558026907000
}}
```

Concurrency Model

The matching engine runs as an asynchronous task:

```rust
tokio::spawn(matching_engine(rx, ws_tx.clone()));
```

REST requests send commands to the engine using a tokio::mpsc channel.

The engine processes commands sequentially but can handle concurrent input efficiently.

Market events are distributed using a tokio::broadcast channel for all connected subscribers.

### OrderBook Model

The matching engine maintains an in-memory OrderBook with efficient data structures:

```rust
pub struct OrderBook {
    bid: BTreeMap<Price, VecDeque<OrderKey>>,
    ask: BTreeMap<Price, VecDeque<OrderKey>>,
    indexed: HashMap<(OrderId, Revision), OrderKey>,
    orders: SlotMap<OrderKey, Order>,
}
```

* BTreeMap keeps bids and asks sorted by price for O(log n) matching.

* Each price level stores a FIFO VecDeque of orders.

* SlotMap provides stable, fast access to orders.

* HashMap allows O(1) lookup for modify/delete operations.

### Benchmarks

The following benchmark measures the time required to insert and match 1,000,000 randomly generated orders:

- Approximately **4.3 million orders per second**.

These times correspond to processing one million order insertions and matches in approximately 230 milliseconds,
depending on hardware and build profile.

### Development

Run locally

```cargo run```

Run tests

```cargo test```

