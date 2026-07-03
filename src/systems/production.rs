//! フェーズ2: 供給会社が部品、製造会社が自転車を生産する（不良品は廃棄）

use bevy::prelude::*;

use crate::constants::OVERSTOCK_HALT_FACTOR;
use crate::model::components::*;
use crate::model::resources::PlayerStats;

/// 在庫が目標の一定倍を超えたら生産を止める（作り過ぎで市場が崩壊するのを防ぐ）
fn overstocked(stock: u32, per_turn: f32, target_turns: u8) -> bool {
    stock as f32 > per_turn * f32::from(target_turns) * OVERSTOCK_HALT_FACTOR
}

#[allow(clippy::type_complexity)]
pub fn produce(
    mut stats: ResMut<PlayerStats>,
    mut q: Query<
        (
            &CompanyKind,
            &mut Wallet,
            &mut Inventory,
            &Facility,
            &TradePolicy,
            Option<&Player>,
        ),
        Without<Bankrupt>,
    >,
) {
    // 前ターンの実績をこのターンの解決開始時にリセット
    *stats = PlayerStats::default();

    for (kind, mut wallet, mut inv, fac, policy, player) in &mut q {
        match kind {
            CompanyKind::Supplier => {
                // 原材料段は省略: 無から生産し、コストは稼働維持費のみ
                if overstocked(inv.parts, fac.speed, policy.target_stock_turns) {
                    continue;
                }
                let n = fac.speed.round().max(0.0) as u32;
                if n == 0 {
                    continue;
                }
                wallet.0 -= (n as f32 * fac.run_cost).round() as i64;
                let good = (n as f32 * (1.0 - fac.defect)).round() as u32;
                inv.parts += good;
            }
            CompanyKind::Manufacturer => {
                if overstocked(inv.bikes, fac.speed, policy.target_stock_turns) {
                    continue;
                }
                // 生産数 = min(製造速度, 部品在庫で作れる数)
                let by_speed = fac.speed.floor().max(0.0) as u32;
                let by_parts = (inv.parts as f32 / fac.efficiency).floor() as u32;
                let n = by_speed.min(by_parts);
                if n == 0 {
                    continue;
                }
                let consumed = (n as f32 * fac.efficiency).ceil() as u32;
                inv.parts = inv.parts.saturating_sub(consumed);
                wallet.0 -= (n as f32 * fac.run_cost).round() as i64;
                let good = (n as f32 * (1.0 - fac.defect)).round() as u32;
                inv.bikes += good;
                if player.is_some() {
                    stats.produced_bikes = good;
                }
            }
            CompanyKind::Retailer => {}
        }
    }
}
