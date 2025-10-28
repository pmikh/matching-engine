use criterion::{Criterion, criterion_group, criterion_main};
use exchange::domain::order_entry::OrderEntry;
use exchange::domain::side::Side;
use exchange::matching::command::MatchingEngineCommand::Create;
use exchange::matching::engine::{MarketEvent, matching_engine};
use rand::Rng;
use std::hint::black_box;
use tokio::sync::{broadcast, mpsc};

fn bench_matching_engine(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut rng = rand::rng();
    let orders: Vec<_> = (0..1_000_000)
        .map(|_| {
            let random_price = rng.random_range(10..150);
            let random_quantity = rng.random_range(1..10);
            let random_side: Side = rng.random();
            Create(OrderEntry::new(random_price, random_quantity, random_side))
        })
        .collect();

    c.bench_function("matching_engine throughput", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let orders = orders.clone();
                let (tx, rx) = mpsc::channel(100_000);
                let (ws_tx, _) = broadcast::channel::<MarketEvent>(1000);

                let engine_handle = tokio::spawn(matching_engine(rx, ws_tx));

                for order in orders {
                    tx.send(black_box(order)).await.unwrap();
                }

                drop(tx);

                engine_handle.await.unwrap();
            })
        });
    });
}

criterion_group!(benches, bench_matching_engine);
criterion_main!(benches);
