use bevy::prelude::*;

use crate::constants::*;

/// 企業の種別。システムの振る舞い分岐に用いる
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompanyKind {
    Supplier,
    Manufacturer,
    Retailer,
}

impl CompanyKind {
    pub fn label_jp(self) -> &'static str {
        match self {
            CompanyKind::Supplier => "部品供給",
            CompanyKind::Manufacturer => "自転車製造",
            CompanyKind::Retailer => "自転車販売",
        }
    }
}

/// 表示用の企業名
#[derive(Component, Clone)]
pub struct CompanyName(pub String);

/// 資金。負になったターン終端で倒産処理
#[derive(Component, Clone, Copy)]
pub struct Wallet(pub i64);

/// 品目別在庫（部品・自転車）
#[derive(Component, Clone, Copy, Default)]
pub struct Inventory {
    pub parts: u32,
    pub bikes: u32,
}

/// 設備の連続値パラメータ（改良で無段階に変化）
#[derive(Component, Clone, Copy, Debug)]
pub struct Facility {
    pub speed: f32,
    pub fixed_cost: f32,
    pub run_cost: f32,
    pub defect: f32,
    pub efficiency: f32,
}

impl Facility {
    pub fn from_array(a: [f32; 5]) -> Self {
        Facility {
            speed: a[0],
            fixed_cost: a[1],
            run_cost: a[2],
            defect: a[3],
            efficiency: a[4],
        }
    }
}

/// 改良対象の設備パラメータ
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FacilityParam {
    Speed,
    FixedCost,
    RunCost,
    Defect,
    Efficiency,
}

impl FacilityParam {
    pub const ALL: [FacilityParam; 5] = [
        FacilityParam::Speed,
        FacilityParam::FixedCost,
        FacilityParam::RunCost,
        FacilityParam::Defect,
        FacilityParam::Efficiency,
    ];

    pub fn index(self) -> usize {
        match self {
            FacilityParam::Speed => 0,
            FacilityParam::FixedCost => 1,
            FacilityParam::RunCost => 2,
            FacilityParam::Defect => 3,
            FacilityParam::Efficiency => 4,
        }
    }

    pub fn label_jp(self) -> &'static str {
        match self {
            FacilityParam::Speed => "製造速度",
            FacilityParam::FixedCost => "固定維持費",
            FacilityParam::RunCost => "稼働維持費",
            FacilityParam::Defect => "不良品率",
            FacilityParam::Efficiency => "資源効率",
        }
    }

    /// 表示用の単位（製造会社視点）
    pub fn unit_jp(self) -> &'static str {
        match self {
            FacilityParam::Speed => "台/ターン",
            FacilityParam::FixedCost => "資金/ターン",
            FacilityParam::RunCost => "資金/台",
            FacilityParam::Defect => "%",
            FacilityParam::Efficiency => "個/台",
        }
    }

    pub fn current(self, f: &Facility) -> f32 {
        match self {
            FacilityParam::Speed => f.speed,
            FacilityParam::FixedCost => f.fixed_cost,
            FacilityParam::RunCost => f.run_cost,
            FacilityParam::Defect => f.defect,
            FacilityParam::Efficiency => f.efficiency,
        }
    }

    /// 改善方向に delta を適用する（速度は加算、他は減算。下限クランプ付き）
    pub fn apply(self, f: &mut Facility, delta: f32) {
        match self {
            FacilityParam::Speed => f.speed += delta,
            FacilityParam::FixedCost => f.fixed_cost = (f.fixed_cost - delta).max(MIN_FIXED_COST),
            FacilityParam::RunCost => f.run_cost = (f.run_cost - delta).max(MIN_RUN_COST),
            FacilityParam::Defect => f.defect = (f.defect - delta).max(MIN_DEFECT),
            FacilityParam::Efficiency => f.efficiency = (f.efficiency - delta).max(MIN_EFFICIENCY),
        }
    }

    /// 初期値からの累積改善度（相対値）。改善が進むほど次の1歩が重くなる
    pub fn improved(self, current: &Facility, baseline: &Facility) -> f32 {
        let cur = self.current(current);
        let base = self.current(baseline);
        if base.abs() < f32::EPSILON {
            return 0.0;
        }
        ((cur - base) / base).abs()
    }

    pub fn base_k(self) -> f32 {
        UPGRADE_BASE_K[self.index()]
    }
}

/// 設備の初期値。改良の逓減計算（improved）の基準
#[derive(Component, Clone, Copy)]
pub struct FacilityBaseline(pub Facility);

/// 進行中の改良プロジェクト（1社1本）
#[derive(Component, Clone, Copy)]
pub struct Upgrade {
    pub target: FacilityParam,
    pub cost_per_turn: i64,
    pub total_turns: u32,
    pub elapsed: u32,
    /// これまでの実績Δの累計（UIで見込みとの乖離を見せる）
    pub accumulated: f32,
}

/// 売買方針。プレイヤーはUI、NPCはAIシステムが書き込む
/// 販売会社では min_sell を消費者向け販売価格 p_i として使う
#[derive(Component, Clone, Copy)]
pub struct TradePolicy {
    pub max_buy: i64,
    pub min_sell: i64,
    pub target_stock_turns: u8,
}

impl TradePolicy {
    pub fn from_tuple(t: (i64, i64, u8)) -> Self {
        TradePolicy {
            max_buy: t.0,
            min_sell: t.1,
            target_stock_turns: t.2,
        }
    }
}

/// プレイヤー企業マーカー
#[derive(Component)]
pub struct Player;

/// NPC性格係数。これを持つ企業はAIが操作する
#[derive(Component, Clone, Copy)]
pub struct AiTraits {
    pub aggressiveness: f32,
    pub invest_bias: f32,
    pub margin: f32,
}

/// 倒産マーカー。全システムから除外し、UIには倒産表示する
#[derive(Component)]
pub struct Bankrupt;

/// 販売会社の直近ターンの販売実績（AIの結果ベース適応に使う）
#[derive(Component, Clone, Copy, Default)]
pub struct SalesStats {
    pub stock_before: u32,
    pub sold: u32,
    /// 在庫切れで需要を取り逃した疑い
    pub sold_out: bool,
}
