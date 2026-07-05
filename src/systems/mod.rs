//! ターン解決のシステムチェーン（§3 のフェーズ順）と世界のセットアップ

pub mod ai;
pub mod bankruptcy;
pub mod market;
pub mod production;
pub mod sales;
pub mod upgrade;
pub mod upkeep;

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::constants::*;
use crate::model::components::*;
use crate::model::resources::*;
use crate::states::GameState;

/// シミュレーション本体。UI 非依存なのでヘッドレステストからも使える
pub struct SimulationPlugin {
    pub seed: u64,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(TurnCounter::default())
            .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(self.seed)))
            .insert_resource(DemandCurve::from_constants(self.seed.wrapping_mul(0x5DEECE66D)))
            .insert_resource(Markets::default())
            .init_resource::<GameResult>()
            .init_resource::<PlayerStats>()
            .init_resource::<PlayerLedger>()
            .add_systems(Startup, setup_companies)
            .add_systems(
                OnEnter(GameState::Resolving),
                (
                    upgrade::advance_upgrades,   // 1. 改良進行
                    production::produce,         // 2. 生産
                    market::clear_parts_market,  // 3. 部品市場約定
                    market::clear_bikes_market,  // 4. 自転車市場約定
                    sales::retail_sales,         // 5. 最終販売
                    upkeep::charge_upkeep,       // 6. 維持費徴収
                    bankruptcy::judge,           // 7. 倒産・勝敗判定
                    ai::npc_decide,              // 8. NPC意思決定
                    record_player_ledger,        // プレイヤー財務指標の集計（UI用）
                    finish_turn,
                )
                    .chain(),
            );
    }
}

/// 初期構成: 供給×3、製造×3（うち1がプレイヤー）、販売×3
fn setup_companies(mut commands: Commands, mut rng: ResMut<GameRng>) {
    let random_traits = |rng: &mut GameRng| AiTraits {
        aggressiveness: rng.0.random_range(AI_AGGRESSIVENESS_RANGE.0..=AI_AGGRESSIVENESS_RANGE.1),
        invest_bias: rng.0.random_range(AI_INVEST_BIAS_RANGE.0..=AI_INVEST_BIAS_RANGE.1),
        margin: rng.0.random_range(AI_MARGIN_RANGE.0..=AI_MARGIN_RANGE.1),
    };

    for i in 0..N_SUPPLIERS {
        let facility = Facility::from_array(SUPPLIER_FACILITY);
        commands.spawn((
            CompanyName(format!("供給{}", i + 1)),
            CompanyKind::Supplier,
            Wallet(INITIAL_FUNDS),
            Inventory::default(),
            facility,
            FacilityBaseline(facility),
            TradePolicy::from_tuple(SUPPLIER_POLICY),
            random_traits(&mut rng),
        ));
    }

    for i in 0..N_MANUFACTURERS {
        let facility = Facility::from_array(MANUFACTURER_FACILITY);
        let is_player = i == 0;
        let name = if is_player {
            "製造1 (あなた)".to_string()
        } else {
            format!("製造{}", i + 1)
        };
        let mut e = commands.spawn((
            CompanyName(name),
            CompanyKind::Manufacturer,
            Wallet(INITIAL_FUNDS),
            Inventory::default(),
            facility,
            FacilityBaseline(facility),
            TradePolicy::from_tuple(MANUFACTURER_POLICY),
        ));
        if is_player {
            e.insert(Player);
        } else {
            let t = random_traits(&mut rng);
            e.insert(t);
        }
    }

    for i in 0..N_RETAILERS {
        let facility = Facility::from_array(RETAILER_FACILITY);
        commands.spawn((
            CompanyName(format!("販売{}", i + 1)),
            CompanyKind::Retailer,
            Wallet(INITIAL_FUNDS),
            Inventory::default(),
            facility,
            FacilityBaseline(facility),
            TradePolicy::from_tuple(RETAILER_POLICY),
            SalesStats::default(),
            random_traits(&mut rng),
        ));
    }
}

/// プレイヤーの累積財務指標を更新する（平均調達・販売価格と資金変動）。UI表示専用
fn record_player_ledger(
    stats: Res<PlayerStats>,
    mut ledger: ResMut<PlayerLedger>,
    player: Query<&Wallet, With<Player>>,
) {
    ledger.parts_qty += stats.bought_parts;
    ledger.parts_cost += stats.parts_cost;
    ledger.bikes_qty += stats.sold_bikes;
    ledger.bikes_revenue += stats.revenue;
    if let Ok(wallet) = player.single() {
        ledger.last_delta = wallet.0 - ledger.prev_wallet;
        ledger.prev_wallet = wallet.0;
    }
}

/// ターン終端: カウンタを進め、勝敗が付いていれば GameOver、なければ入力待ちへ戻す
fn finish_turn(
    mut turn: ResMut<TurnCounter>,
    result: Res<GameResult>,
    mut next: ResMut<NextState<GameState>>,
) {
    turn.0 += 1;
    if result.0.is_some() {
        next.set(GameState::GameOver);
    } else {
        next.set(GameState::AwaitingInput);
    }
}
