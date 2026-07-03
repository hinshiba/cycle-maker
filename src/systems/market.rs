//! フェーズ3・4: 部品市場と自転車市場の板寄せ約定
//! 注文数量は TradePolicy（指値と目標在庫ターン数）から自動算出する（§7）

use bevy::prelude::*;

use crate::core::auction::{clear, Order, Trade};
use crate::model::components::*;
use crate::model::market::Good;
use crate::model::resources::{Markets, PlayerStats};

type CompanyQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static CompanyKind,
        &'static mut Wallet,
        &'static mut Inventory,
        &'static TradePolicy,
        &'static Facility,
        Option<&'static SalesStats>,
        Option<&'static Player>,
    ),
    Without<Bankrupt>,
>;

/// 約定を資金・在庫に反映する
fn settle(
    trades: &[Trade<Entity>],
    price: i64,
    good: Good,
    companies: &mut CompanyQuery,
    player_stats: &mut PlayerStats,
) {
    for t in trades {
        let total = price * i64::from(t.qty);
        if let Ok((_, _, mut wallet, mut inv, _, _, _, player)) = companies.get_mut(t.seller) {
            wallet.0 += total;
            match good {
                Good::Parts => inv.parts = inv.parts.saturating_sub(t.qty),
                Good::Bikes => inv.bikes = inv.bikes.saturating_sub(t.qty),
            }
            if player.is_some() && good == Good::Bikes {
                player_stats.sold_bikes += t.qty;
                player_stats.revenue += total;
            }
        }
        if let Ok((_, _, mut wallet, mut inv, _, _, _, _)) = companies.get_mut(t.buyer) {
            wallet.0 -= total;
            match good {
                Good::Parts => inv.parts += t.qty,
                Good::Bikes => inv.bikes += t.qty,
            }
        }
    }
}

/// 資金と指値から買える上限数量
fn affordable_qty(wallet: i64, price: i64, want: u32) -> u32 {
    if wallet <= 0 || price <= 0 {
        return 0;
    }
    want.min((wallet / price).min(i64::from(u32::MAX)) as u32)
}

/// 部品市場: 供給(売) × 製造(買)
pub fn clear_parts_market(
    mut markets: ResMut<Markets>,
    mut stats: ResMut<PlayerStats>,
    mut companies: CompanyQuery,
) {
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for (entity, kind, wallet, inv, policy, fac, _, _) in companies.iter() {
        match kind {
            CompanyKind::Supplier => {
                if inv.parts > 0 {
                    asks.push(Order {
                        agent: entity,
                        price: policy.min_sell,
                        qty: inv.parts,
                    });
                }
            }
            CompanyKind::Manufacturer => {
                // 目標在庫 = 1ターンの部品消費量 × 目標在庫ターン数
                let per_turn = (fac.speed * fac.efficiency).ceil() as u32;
                let target = per_turn * u32::from(policy.target_stock_turns);
                let want = target.saturating_sub(inv.parts);
                let qty = affordable_qty(wallet.0, policy.max_buy, want);
                if qty > 0 {
                    bids.push(Order {
                        agent: entity,
                        price: policy.max_buy,
                        qty,
                    });
                }
            }
            CompanyKind::Retailer => {}
        }
    }

    if let Some(clearing) = clear(&bids, &asks) {
        markets.parts.last_price = clearing.price;
        markets.parts.volume = clearing.volume;
        settle(&clearing.trades, clearing.price, Good::Parts, &mut companies, &mut stats);
    } else {
        markets.parts.volume = 0;
    }
}

/// 自転車市場: 製造(売) × 販売(買)
pub fn clear_bikes_market(
    mut markets: ResMut<Markets>,
    mut stats: ResMut<PlayerStats>,
    mut companies: CompanyQuery,
) {
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for (entity, kind, wallet, inv, policy, _, sales, _) in companies.iter() {
        match kind {
            CompanyKind::Manufacturer => {
                if inv.bikes > 0 {
                    asks.push(Order {
                        agent: entity,
                        price: policy.min_sell,
                        qty: inv.bikes,
                    });
                }
            }
            CompanyKind::Retailer => {
                // 目標在庫 = 直近の販売実績 × 目標在庫ターン数（実績ゼロ時は最低ラインを見込む）
                let expected_sales = sales.map(|s| s.sold).unwrap_or(0).max(5);
                let target = expected_sales * u32::from(policy.target_stock_turns);
                let want = target.saturating_sub(inv.bikes);
                let qty = affordable_qty(wallet.0, policy.max_buy, want);
                if qty > 0 {
                    bids.push(Order {
                        agent: entity,
                        price: policy.max_buy,
                        qty,
                    });
                }
            }
            CompanyKind::Supplier => {}
        }
    }

    if let Some(clearing) = clear(&bids, &asks) {
        markets.bikes.last_price = clearing.price;
        markets.bikes.volume = clearing.volume;
        settle(&clearing.trades, clearing.price, Good::Bikes, &mut companies, &mut stats);
    } else {
        markets.bikes.volume = 0;
    }
}
