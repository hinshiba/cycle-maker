//! コールオークション（板寄せ）の純粋関数実装

/// 1件の注文。Id は呼び出し側の識別子（本体では Entity、テストでは u32）
#[derive(Clone, Debug, PartialEq)]
pub struct Order<Id> {
    pub agent: Id,
    pub price: i64,
    pub qty: u32,
}

/// 約定1件。価格は板全体で一様（Clearing::price）
#[derive(Clone, Debug, PartialEq)]
pub struct Trade<Id> {
    pub buyer: Id,
    pub seller: Id,
    pub qty: u32,
}

/// 板寄せの結果
#[derive(Clone, Debug, PartialEq)]
pub struct Clearing<Id> {
    pub price: i64,
    pub volume: u32,
    pub trades: Vec<Trade<Id>>,
}

/// 板寄せ: 買いを価格降順・売りを価格昇順に並べ、交差する範囲を最大量マッチングする。
/// 約定価格は限界（最後に成立した）買値と売値の中間値。約定ゼロなら None
pub fn clear<Id: Copy>(bids: &[Order<Id>], asks: &[Order<Id>]) -> Option<Clearing<Id>> {
    let mut bids: Vec<Order<Id>> = bids.iter().filter(|o| o.qty > 0 && o.price > 0).cloned().collect();
    let mut asks: Vec<Order<Id>> = asks.iter().filter(|o| o.qty > 0 && o.price > 0).cloned().collect();
    bids.sort_by_key(|o| std::cmp::Reverse(o.price));
    asks.sort_by_key(|o| o.price);

    let mut trades = Vec::new();
    let mut volume: u32 = 0;
    let (mut marginal_bid, mut marginal_ask) = (0i64, 0i64);
    let (mut bi, mut ai) = (0usize, 0usize);

    while bi < bids.len() && ai < asks.len() {
        if bids[bi].price < asks[ai].price {
            break;
        }
        let qty = bids[bi].qty.min(asks[ai].qty);
        trades.push(Trade {
            buyer: bids[bi].agent,
            seller: asks[ai].agent,
            qty,
        });
        volume += qty;
        marginal_bid = bids[bi].price;
        marginal_ask = asks[ai].price;
        bids[bi].qty -= qty;
        asks[ai].qty -= qty;
        if bids[bi].qty == 0 {
            bi += 1;
        }
        if asks[ai].qty == 0 {
            ai += 1;
        }
    }

    if volume == 0 {
        return None;
    }
    Some(Clearing {
        price: (marginal_bid + marginal_ask) / 2,
        volume,
        trades,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(agent: u32, price: i64, qty: u32) -> Order<u32> {
        Order { agent, price, qty }
    }

    #[test]
    fn no_cross_returns_none() {
        // 買値が売値に届かない → 約定なし
        assert_eq!(clear(&[o(1, 40, 10)], &[o(2, 50, 10)]), None);
    }

    #[test]
    fn empty_book_returns_none() {
        assert_eq!(clear::<u32>(&[], &[o(2, 50, 10)]), None);
        assert_eq!(clear::<u32>(&[o(1, 50, 10)], &[]), None);
    }

    #[test]
    fn exact_match_clears_at_midpoint() {
        let c = clear(&[o(1, 100, 10)], &[o(2, 90, 10)]).unwrap();
        assert_eq!(c.volume, 10);
        assert_eq!(c.price, 95);
        assert_eq!(c.trades, vec![Trade { buyer: 1, seller: 2, qty: 10 }]);
    }

    #[test]
    fn partial_fill_leaves_remainder_unmatched() {
        // 買い5に対し売り10 → 5だけ約定
        let c = clear(&[o(1, 100, 5)], &[o(2, 90, 10)]).unwrap();
        assert_eq!(c.volume, 5);
        assert_eq!(c.trades, vec![Trade { buyer: 1, seller: 2, qty: 5 }]);
    }

    #[test]
    fn multiple_orders_match_in_price_priority() {
        // 買い: 120x5, 100x5 / 売り: 80x4, 110x4
        // 120 vs 80 で4約定、120残1 vs 110 で1約定、100 vs 110 は交差せず終了
        let c = clear(
            &[o(1, 120, 5), o(2, 100, 5)],
            &[o(3, 80, 4), o(4, 110, 4)],
        )
        .unwrap();
        assert_eq!(c.volume, 5);
        assert_eq!(
            c.trades,
            vec![
                Trade { buyer: 1, seller: 3, qty: 4 },
                Trade { buyer: 1, seller: 4, qty: 1 },
            ]
        );
        // 限界ペアは (120, 110) → 中間値 115
        assert_eq!(c.price, 115);
    }

    #[test]
    fn zero_qty_orders_are_ignored() {
        assert_eq!(clear(&[o(1, 100, 0)], &[o(2, 90, 10)]), None);
    }
}
