use bevy::prelude::*;
use rand_chacha::ChaCha8Rng;

use crate::constants::*;
use crate::model::market::{Good, MarketState};

/// 経過ターン数
#[derive(Resource, Default)]
pub struct TurnCounter(pub u32);

/// シード固定可能なゲーム全体の乱数源（テスト再現性のため）
#[derive(Resource)]
pub struct GameRng(pub ChaCha8Rng);

/// 隠蔽された基礎需要 D(turn)。sales システム以外から参照禁止（エージェントには見せない）
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
    /// 目標資本金に到達（到達ターン数を記録）
    Victory { turn: u32 },
    /// プレイヤー倒産
    Bankrupt { turn: u32 },
}

#[derive(Resource, Default)]
pub struct GameResult(pub Option<Outcome>);

/// プレイヤーの直近ターンの販売実績（自転車市場での約定）
#[derive(Resource, Default, Clone, Copy)]
pub struct PlayerStats {
    pub sold_bikes: u32,
    pub revenue: i64,
    pub produced_bikes: u32,
}
