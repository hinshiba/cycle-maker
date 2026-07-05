use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::constants::*;
use crate::model::market::{Good, MarketState};

/// 経過ターン数
#[derive(Resource, Default)]
pub struct TurnCounter(pub u32);

/// シード固定可能なゲーム全体の乱数源
#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

/// 需要 D(turn) -> sales システム以外から参照禁止（エージェントには見せない）
#[derive(Resource)]
pub struct DemandCurve {
    pub base: f32,
    pub trend: f32,
    pub week_amp: f32,
    pub noise_amp: f32,
    pub min: f32,
    pub noise_seed: u64,
}

impl DemandCurve {
    pub fn from_constants(noise_seed: u64) -> Self {
        DemandCurve {
            base: DEMAND_BASE,
            trend: DEMAND_TREND,
            week_amp: DEMAND_WEEK_AMP,
            noise_amp: DEMAND_NOISE_AMP,
            min: DEMAND_MIN,
            noise_seed,
        }
    }
}

/// 部品市場と自転車市場
#[derive(Resource)]
pub struct Markets {
    pub parts: MarketState,
    pub bikes: MarketState,
}

impl Default for Markets {
    fn default() -> Self {
        Markets {
            parts: MarketState::new(Good::Parts),
            bikes: MarketState::new(Good::Bikes),
        }
    }
}

/// 勝敗の結果
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// 目標資本金に到達
    Victory { turn: u32 },
    /// プレイヤー倒産
    Bankrupt { turn: u32 },
}

#[derive(Resource, Default)]
pub struct GameResult(pub Option<Outcome>);

/// プレイヤーの直近ターンの実績（毎ターン生産フェーズでリセットする）
#[derive(Resource, Default, Clone, Copy)]
pub struct PlayerStats {
    pub sold_bikes: u32,
    pub revenue: i64,
    pub produced_bikes: u32,
    /// 部品市場で買い付けた数量と支払額
    pub bought_parts: u32,
    pub parts_cost: i64,
}

/// プレイヤーの累積財務指標（UI表示専用）。ターンをまたいで保持する
#[derive(Resource, Clone, Copy)]
pub struct PlayerLedger {
    /// 直近解決ターンでの資金変動
    pub last_delta: i64,
    /// 資金変動の算出に使う前ターン終端の資金
    pub prev_wallet: i64,
    /// 部品の累計調達量と累計支払（平均調達価格の算出用）
    pub parts_qty: u32,
    pub parts_cost: i64,
    /// 自転車の累計販売量と累計売上（平均販売価格の算出用）
    pub bikes_qty: u32,
    pub bikes_revenue: i64,
}

impl Default for PlayerLedger {
    fn default() -> Self {
        PlayerLedger {
            last_delta: 0,
            prev_wallet: INITIAL_FUNDS,
            parts_qty: 0,
            parts_cost: 0,
            bikes_qty: 0,
            bikes_revenue: 0,
        }
    }
}

impl PlayerLedger {
    /// 平均部品調達価格（未取引なら None）
    pub fn avg_parts_price(&self) -> Option<f32> {
        (self.parts_qty > 0).then(|| self.parts_cost as f32 / self.parts_qty as f32)
    }

    /// 平均販売価格（未取引なら None）
    pub fn avg_bike_price(&self) -> Option<f32> {
        (self.bikes_qty > 0).then(|| self.bikes_revenue as f32 / self.bikes_qty as f32)
    }
}
