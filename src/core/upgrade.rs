//! 改良プロジェクトの成果Δ計算の純粋関数（§6）
//! Δ = ε * f(v_k, k, c) * g(t, T)

use crate::constants::{UPGRADE_COST_EXP, UPGRADE_IMPROVE_PENALTY};

/// 費用効果 f = base_k * c^0.7 / (1 + penalty * improved)
/// 費用の限界効果は逓減し、改善が進むほど次の1歩が重くなる
pub fn cost_effect(base_k: f32, cost_per_turn: f32, improved: f32) -> f32 {
    if cost_per_turn <= 0.0 {
        return 0.0;
    }
    base_k * cost_per_turn.powf(UPGRADE_COST_EXP) / (1.0 + UPGRADE_IMPROVE_PENALTY * improved)
}

/// 期間内進捗カーブ g = 4t(T-t)/T²（山型: 序盤鈍く中盤に伸び終盤収束）
pub fn progress_curve(t: f32, total: f32) -> f32 {
    if total <= 0.0 {
        return 0.0;
    }
    (4.0 * t * (total - t) / (total * total)).max(0.0)
}

/// あるターンの見込みΔ（ε の期待値 1 とした値）。UIの見込み表示に使う
pub fn expected_delta(base_k: f32, cost_per_turn: f32, improved: f32, t: f32, total: f32) -> f32 {
    cost_effect(base_k, cost_per_turn, improved) * progress_curve(t, total)
}

/// あるターンの実績Δ。ε は呼び出し側が U[0.5, 1.5] で引く
pub fn upgrade_delta(
    base_k: f32,
    cost_per_turn: f32,
    improved: f32,
    t: f32,
    total: f32,
    eps: f32,
) -> f32 {
    expected_delta(base_k, cost_per_turn, improved, t, total) * eps
}

/// プロジェクト全期間の見込みΔ合計（improved は開始時点で固定した近似）
pub fn expected_total(base_k: f32, cost_per_turn: f32, improved: f32, total_turns: u32) -> f32 {
    let total = total_turns as f32;
    (0..total_turns)
        .map(|t| expected_delta(base_k, cost_per_turn, improved, t as f32 + 0.5, total))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_curve_is_bell_shaped() {
        // 中央で最大、端で小さい山型
        let total = 10.0;
        let mid = progress_curve(5.0, total);
        let early = progress_curve(0.5, total);
        let late = progress_curve(9.5, total);
        assert!(mid > early);
        assert!(mid > late);
        assert!((mid - 1.0).abs() < 1e-6); // 頂点 g(T/2) = 1
        assert!(early > 0.0);
    }

    #[test]
    fn cost_has_diminishing_returns() {
        // 費用2倍でも効果は2倍未満（c^0.7）
        let f1 = cost_effect(0.01, 100.0, 0.0);
        let f2 = cost_effect(0.01, 200.0, 0.0);
        assert!(f2 > f1);
        assert!(f2 < 2.0 * f1);
    }

    #[test]
    fn improvement_makes_next_step_heavier() {
        let fresh = cost_effect(0.01, 100.0, 0.0);
        let improved = cost_effect(0.01, 100.0, 0.5);
        assert!(improved < fresh);
    }

    #[test]
    fn zero_cost_yields_zero_delta() {
        assert_eq!(expected_delta(0.01, 0.0, 0.0, 5.0, 10.0), 0.0);
    }

    #[test]
    fn eps_scales_delta_linearly() {
        let e = expected_delta(0.01, 100.0, 0.0, 5.0, 10.0);
        let lo = upgrade_delta(0.01, 100.0, 0.0, 5.0, 10.0, 0.5);
        let hi = upgrade_delta(0.01, 100.0, 0.0, 5.0, 10.0, 1.5);
        assert!((lo - e * 0.5).abs() < 1e-6);
        assert!((hi - e * 1.5).abs() < 1e-6);
    }

    #[test]
    fn expected_total_grows_with_cost() {
        let cheap = expected_total(0.01, 50.0, 0.0, 10);
        let rich = expected_total(0.01, 500.0, 0.0, 10);
        assert!(rich > cheap);
        assert!(cheap > 0.0);
    }
}
