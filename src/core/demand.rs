//! 隠蔽需要 D(turn) と販売会社の販売量の純粋関数（§5）

use crate::model::resources::DemandCurve;

/// ターン番号から決定的に [-1, 1] のノイズを得る（splitmix64 ベースのハッシュ）
pub fn noise(seed: u64, turn: u32) -> f32 {
    let mut x = seed ^ (u64::from(turn).wrapping_add(1)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
}

/// 基礎需要 D(turn) = 緩やかなトレンド + 週周期 + ノイズ。下限クランプ付き（連鎖崩壊の安全弁）
pub fn base_demand(curve: &DemandCurve, turn: u32) -> f32 {
    let t = turn as f32;
    let weekly = curve.week_amp * (t * std::f32::consts::TAU / 7.0).sin();
    let n = curve.noise_amp * noise(curve.noise_seed, turn);
    (curve.base + curve.trend * t + weekly + n).max(curve.min)
}

/// 販売会社1社が需要関数で捌ける量: (D/N_ret) * (P_ref/p)^e
pub fn retail_demand(d: f32, n_ret: usize, price: f32, p_ref: f32, elasticity: f32) -> f32 {
    if n_ret == 0 || price <= 0.0 {
        return 0.0;
    }
    (d / n_ret as f32) * (p_ref / price).powf(elasticity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve() -> DemandCurve {
        DemandCurve {
            base: 40.0,
            trend: 0.02,
            week_amp: 8.0,
            noise_amp: 6.0,
            min: 15.0,
            noise_seed: 1,
        }
    }

    #[test]
    fn demand_is_clamped_to_minimum() {
        let mut c = curve();
        c.base = -100.0;
        for turn in 0..50 {
            assert!(base_demand(&c, turn) >= c.min);
        }
    }

    #[test]
    fn demand_is_deterministic_per_turn() {
        let c = curve();
        assert_eq!(base_demand(&c, 10), base_demand(&c, 10));
    }

    #[test]
    fn higher_price_sells_less() {
        let cheap = retail_demand(60.0, 3, 200.0, 300.0, 1.5);
        let expensive = retail_demand(60.0, 3, 400.0, 300.0, 1.5);
        assert!(cheap > expensive);
    }

    #[test]
    fn reference_price_sells_exact_share() {
        // p = P_ref なら D/N がそのまま出る
        let q = retail_demand(60.0, 3, 300.0, 300.0, 1.5);
        assert!((q - 20.0).abs() < 1e-4);
    }

    #[test]
    fn zero_retailers_or_price_sells_nothing() {
        assert_eq!(retail_demand(60.0, 0, 300.0, 300.0, 1.5), 0.0);
        assert_eq!(retail_demand(60.0, 3, 0.0, 300.0, 1.5), 0.0);
    }
}
