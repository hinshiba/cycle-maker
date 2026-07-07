use crate::constants::*;

/// 取引品目
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Good {
    Parts,
    Bikes,
}

/// 1市場の公開状態（直近約定価格と出来高）。板は毎ターン組み直すため保持しない
#[derive(Clone, Copy, Debug)]
pub struct MarketState {
    pub good: Good,
    pub last_price: i64,
    pub volume: u32,
}

impl MarketState {
    pub fn new(good: Good) -> Self {
        let last_price = match good {
            Good::Parts => INIT_PARTS_PRICE,
            Good::Bikes => INIT_BIKE_PRICE,
        };
        MarketState {
            good,
            last_price,
            volume: 0,
        }
    }
}
