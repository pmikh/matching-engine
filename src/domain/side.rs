use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum Side {
    Buy,
    Sell,
}

impl Distribution<Side> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.random_range(0..=2) {
            0 => Side::Buy,
            1 => Side::Sell,
            _ => Side::Buy,
        }
    }
}
