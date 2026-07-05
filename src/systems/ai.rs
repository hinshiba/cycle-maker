//! フェーズ8: NPC意思決定（ルールベース、§8）
//! 需要関数・乱数の内部は参照せず、観測結果（自社実績と直近約定価格）のみから適応する

use bevy::prelude::*;
use rand::RngExt;

use crate::constants::*;
use crate::core::upgrade::expected_total;
use crate::model::components::*;
use crate::model::resources::{GameRng, Markets};

/// 業種ごとに改良して意味のあるパラメータ
fn allowed_params(kind: CompanyKind) -> &'static [FacilityParam] {
    match kind {
        CompanyKind::Supplier => &[
            FacilityParam::Speed,
            FacilityParam::FixedCost,
            FacilityParam::RunCost,
            FacilityParam::Defect,
        ],
        CompanyKind::Manufacturer => &FacilityParam::ALL,
        CompanyKind::Retailer => &[FacilityParam::FixedCost],
    }
}

#[allow(clippy::type_complexity)]
pub fn npc_decide(
    mut commands: Commands,
    markets: Res<Markets>,
    mut rng: ResMut<GameRng>,
    mut q: Query<
        (
            Entity,
            &CompanyKind,
            &Wallet,
            &Inventory,
            &Facility,
            &FacilityBaseline,
            &mut TradePolicy,
            Option<&SalesStats>,
            &AiTraits,
            Option<&Upgrade>,
        ),
        (With<AiTraits>, Without<Bankrupt>),
    >,
) {
    let last_parts = markets.parts.last_price.max(1);
    let last_bikes = markets.bikes.last_price.max(1);

    for (entity, kind, wallet, inv, fac, baseline, mut policy, sales, traits, upgrade) in &mut q {
        let k = AI_PRICE_ADJUST_K * traits.aggressiveness;

        match kind {
            CompanyKind::Supplier => {
                // 在庫乖離率で売り指値を上下: 在庫過剰なら値下げ、品薄なら値上げ
                let target = (fac.speed * f32::from(policy.target_stock_turns)).max(1.0);
                let dev = (inv.parts as f32 - target) / target;
                // 固定費の按分まで含めた実質原価を割らない
                let unit_cost = fac.run_cost + fac.fixed_cost / fac.speed.max(1.0);
                let floor = ((unit_cost * 1.05) as i64).max(1);
                policy.min_sell =
                    ((last_parts as f32 * (1.0 - k * dev.clamp(-1.0, 1.0))) as i64).max(floor);
            }
            CompanyKind::Manufacturer => {
                // 部品の買い指値: 在庫不足なら強気に
                let per_turn = (fac.speed * fac.efficiency).max(1.0);
                let target = per_turn * f32::from(policy.target_stock_turns);
                let dev_parts = (inv.parts as f32 - target) / target;
                policy.max_buy =
                    ((last_parts as f32 * (1.0 - k * dev_parts.clamp(-1.0, 1.0))) as i64).max(1);

                // 自転車の売り指値: 在庫過剰なら値下げ。原価割れはしない
                let bike_target = (fac.speed * f32::from(policy.target_stock_turns)).max(1.0);
                let dev_bikes = (inv.bikes as f32 - bike_target) / bike_target;
                let unit_cost = fac.efficiency * last_parts as f32 + fac.run_cost;
                let floor = (unit_cost * 1.02) as i64;
                policy.min_sell = ((last_bikes as f32 * (1.0 - k * dev_bikes.clamp(-1.0, 1.0)))
                    as i64)
                    .max(floor);
            }
            CompanyKind::Retailer => {
                // 結果からのルールベース適応（需要関数は読めない前提）
                let delta = AI_RETAIL_DELTA * traits.aggressiveness;
                if let Some(s) = sales {
                    if s.sold_out {
                        // 在庫0で需要を取り逃した疑い → 値上げ・買付量↑
                        // （仕入れられない場合もこの値上げが買付上限を押し上げ、いずれ約定に至る）
                        policy.min_sell = ((policy.min_sell as f32 * (1.0 + delta)) as i64).max(1);
                        policy.target_stock_turns = (policy.target_stock_turns + 1).min(6);
                    } else {
                        let leftover = (s.stock_before - s.sold) as f32 / s.stock_before as f32;
                        if leftover > AI_LEFTOVER_THRESHOLD {
                            // 売れ残り超過 → 値下げ・買付量↓
                            policy.min_sell =
                                ((policy.min_sell as f32 * (1.0 - delta)) as i64).max(1);
                            policy.target_stock_turns =
                                policy.target_stock_turns.saturating_sub(1).max(1);
                        }
                    }
                }
                // 買付指値の上限 = 現販売価格 − 目標マージン
                policy.max_buy = ((policy.min_sell as f32 * (1.0 - traits.margin)) as i64).max(1);
            }
        }

        // 改良判断: 資金が閾値超過かつ改良中でなければ、粗ROI最大の要素へ投資
        if upgrade.is_none() && wallet.0 > AI_INVEST_WEALTH_THRESHOLD {
            let cost = ((wallet.0 / 200).clamp(50, 500) as f32 * traits.invest_bias) as i64;
            let turns = AI_UPGRADE_TURNS;
            let mut best: Option<(FacilityParam, f32)> = None;
            for &param in allowed_params(*kind) {
                let improved = param.improved(fac, &baseline.0);
                let gain = expected_total(param.base_k(), cost as f32, improved, turns);
                // 粗ROI = 改善量 × 金額換算 × 想定回収期間 − 投資総額
                let score = gain * AI_ROI_WEIGHT[param.index()] * AI_ROI_HORIZON
                    - (cost * i64::from(turns)) as f32;
                if score > 0.0 && best.map(|(_, s)| score > s).unwrap_or(true) {
                    best = Some((param, score));
                }
            }
            if let Some((param, _)) = best {
                // 毎ターン確実にではなく、投資性向に応じた確率で着手する
                if rng.0.random_range(0.0..1.0) < 0.3 * traits.invest_bias {
                    commands.entity(entity).insert(Upgrade {
                        target: param,
                        cost_per_turn: cost,
                        total_turns: turns,
                        elapsed: 0,
                        accumulated: 0.0,
                    });
                }
            }
        }
    }
}
