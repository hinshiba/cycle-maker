//! 全バランス係数を1箇所に集約する（仕様 §14）。
//! 数値はすべて調整前提の仮値。ヘッドレスバランステスト（tests/balance.rs）で検証する。

// ---- 資金・勝敗 ----
pub const INITIAL_FUNDS: i64 = 100_000;
/// 勝利条件: 初期資金の10倍
pub const GOAL_FUNDS: i64 = 1_000_000;

// ---- 企業構成 ----
pub const N_SUPPLIERS: usize = 3;
pub const N_MANUFACTURERS: usize = 3; // うち1つがプレイヤー
pub const N_RETAILERS: usize = 3;

// ---- 設備初期値（業種別） ----
// Facility { speed, fixed_cost, run_cost, defect, efficiency }
pub const SUPPLIER_FACILITY: [f32; 5] = [38.0, 300.0, 5.0, 0.05, 1.0];
pub const MANUFACTURER_FACILITY: [f32; 5] = [12.0, 400.0, 30.0, 0.08, 3.0];
pub const RETAILER_FACILITY: [f32; 5] = [0.0, 300.0, 0.0, 0.0, 1.0];

// ---- 設備パラメータの下限 ----
pub const MIN_DEFECT: f32 = 0.01;
pub const MIN_EFFICIENCY: f32 = 2.0;
pub const MIN_FIXED_COST: f32 = 20.0;
pub const MIN_RUN_COST: f32 = 0.5;

/// 在庫が目標の何倍を超えたら生産を止めるか（作り過ぎによる価格崩壊の抑制）
pub const OVERSTOCK_HALT_FACTOR: f32 = 3.0;

// ---- 市場初期値 ----
pub const INIT_PARTS_PRICE: i64 = 50;
pub const INIT_BIKE_PRICE: i64 = 250;

// ---- 売買方針の初期値（業種別: max_buy, min_sell, target_stock_turns） ----
pub const SUPPLIER_POLICY: (i64, i64, u8) = (0, 45, 2);
pub const MANUFACTURER_POLICY: (i64, i64, u8) = (55, 240, 2);
pub const RETAILER_POLICY: (i64, i64, u8) = (250, 330, 2);

// ---- 需要モデル（§5）: D(turn) は全エージェントから隠蔽 ----
pub const DEMAND_BASE: f32 = 40.0;
pub const DEMAND_TREND: f32 = 0.02;
/// 週周期（7ターン）の振幅
pub const DEMAND_WEEK_AMP: f32 = 8.0;
pub const DEMAND_NOISE_AMP: f32 = 6.0;
/// 安全弁: 連鎖崩壊防止のための基礎需要下限
pub const DEMAND_MIN: f32 = 15.0;
/// 需要関数の基準価格 P_ref
pub const PRICE_REF: f32 = 300.0;
/// 価格弾力性 e
pub const ELASTICITY: f32 = 1.5;

// ---- 改良プロジェクト（§6） ----
/// ε ~ U[EPS_MIN, EPS_MAX]
pub const UPGRADE_EPS_MIN: f32 = 0.5;
pub const UPGRADE_EPS_MAX: f32 = 1.5;
/// 費用の限界効果逓減 c^0.7
pub const UPGRADE_COST_EXP: f32 = 0.7;
/// 累積改善度による逓減の強さ: f ∝ 1/(1 + PENALTY * improved)
pub const UPGRADE_IMPROVE_PENALTY: f32 = 3.0;
/// 費用効果の基準係数 base_k（パラメータの桁に合わせたスケール）
/// [speed, fixed_cost, run_cost, defect, efficiency]
pub const UPGRADE_BASE_K: [f32; 5] = [0.012, 0.24, 0.024, 0.000_24, 0.002_4];

// ---- プレイヤーUIのスライダ範囲 ----
pub const UI_MAX_BUY_RANGE: (i64, i64) = (0, 200);
pub const UI_MIN_SELL_RANGE: (i64, i64) = (0, 1000);
pub const UI_STOCK_TURNS_RANGE: (u8, u8) = (1, 10);
pub const UI_UPGRADE_COST_RANGE: (i64, i64) = (10, 2000);
pub const UI_UPGRADE_TURNS_RANGE: (u32, u32) = (2, 30);

// ---- NPC AI（§8） ----
/// 在庫乖離率に対する指値調整係数 k
pub const AI_PRICE_ADJUST_K: f32 = 0.2;
/// 販売会社の価格調整幅 δ（強気度で変調）
pub const AI_RETAIL_DELTA: f32 = 0.05;
/// 売れ残り率がこれを超えたら値下げ
pub const AI_LEFTOVER_THRESHOLD: f32 = 0.5;
/// 資金がこれを超えたら改良投資を検討
pub const AI_INVEST_WEALTH_THRESHOLD: i64 = 150_000;
/// 粗ROI見積り: 1単位改善の1ターンあたり価値（金額換算）
/// [speed, fixed_cost, run_cost, defect, efficiency]
pub const AI_ROI_WEIGHT: [f32; 5] = [60.0, 1.0, 12.0, 3000.0, 600.0];
/// ROI見積りの想定回収ターン数
pub const AI_ROI_HORIZON: f32 = 50.0;
/// NPC改良プロジェクトの標準期間
pub const AI_UPGRADE_TURNS: u32 = 8;

// ---- AI個体差の範囲（P0では控えめな分散） ----
pub const AI_AGGRESSIVENESS_RANGE: (f32, f32) = (0.8, 1.2);
pub const AI_INVEST_BIAS_RANGE: (f32, f32) = (0.8, 1.2);
pub const AI_MARGIN_RANGE: (f32, f32) = (0.15, 0.30);
