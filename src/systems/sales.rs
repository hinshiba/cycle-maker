//! フェーズ5: 最終販売。販売会社の在庫が隠蔽需要関数に従って消滅し、売上が立つ（§5）
//! DemandCurve を参照してよいのはこのシステムだけ（エージェントからは隠蔽）

use bevy::prelude::*;

use crate::constants::{ELASTICITY, PRICE_REF};
use crate::core::demand::{base_demand, retail_demand};
use crate::model::components::*;
use crate::model::resources::{DemandCurve, TurnCounter};

pub fn retail_sales(
    curve: Res<DemandCurve>,
    turn: Res<TurnCounter>,
    mut q: Query<
        (
            &CompanyKind,
            &mut Wallet,
            &mut Inventory,
            &TradePolicy,
            &mut SalesStats,
        ),
        Without<Bankrupt>,
    >,
) {
    let n_ret = q
        .iter()
        .filter(|(kind, ..)| **kind == CompanyKind::Retailer)
        .count();
    if n_ret == 0 {
        return;
    }
    let d = base_demand(&curve, turn.0);

    for (kind, mut wallet, mut inv, policy, mut stats) in &mut q {
        if *kind != CompanyKind::Retailer {
            continue;
        }
        // min_sell = 消費者向け販売価格 p_i
        let price = policy.min_sell.max(1) as f32;
        let demand_qty = retail_demand(d, n_ret, price, PRICE_REF, ELASTICITY).round() as u32;
        let sold = demand_qty.min(inv.bikes);

        *stats = SalesStats {
            stock_before: inv.bikes,
            sold,
            sold_out: demand_qty > inv.bikes,
        };
        inv.bikes -= sold;
        wallet.0 += i64::from(sold) * policy.min_sell;
    }
}
