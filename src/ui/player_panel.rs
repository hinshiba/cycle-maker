//! 左パネル: 自社（プレイヤー製造会社）
//! 表示順は 財務 → 購入 → 売却 → 設備 → 改良

use bevy::prelude::*;
use bevy_egui::egui;

use super::PlayerQuery;
use crate::constants::*;
use crate::core::upgrade::{expected_delta, expected_total};
use crate::model::components::*;
use crate::model::resources::{Markets, PlayerLedger};

/// 改良開始前のUI入力値
pub struct UpgradeDraft {
    pub param: FacilityParam,
    pub cost: i64,
    pub turns: u32,
}

impl Default for UpgradeDraft {
    fn default() -> Self {
        UpgradeDraft {
            param: FacilityParam::Speed,
            cost: 200,
            turns: 8,
        }
    }
}

pub fn draw(
    root: &mut egui::Ui,
    commands: &mut Commands,
    draft: &mut UpgradeDraft,
    player: &mut PlayerQuery,
    ledger: &PlayerLedger,
    markets: &Markets,
) {
    let Ok((entity, wallet, inv, fac, baseline, mut policy, upgrade)) = player.single_mut() else {
        return;
    };

    egui::Panel::left("player_panel")
        .default_size(310.0)
        .show(root, |ui| {
            ui.heading("自社（自転車製造）");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                draw_finance(ui, wallet, ledger);
                ui.separator();
                draw_buy(ui, inv, &mut policy, ledger);
                ui.separator();
                draw_sell(ui, inv, &mut policy, ledger);
                ui.separator();
                draw_facility(ui, fac, markets);
                ui.separator();
                draw_upgrade(ui, commands, draft, entity, fac, &baseline, upgrade);
            });
        });
}

fn draw_finance(ui: &mut egui::Ui, wallet: &Wallet, ledger: &PlayerLedger) {
    ui.strong("財務");
    ui.label(format!("現在の資金: {} 円", wallet.0));

    let d = ledger.last_delta;
    let (color, sign) = if d >= 0 {
        (egui::Color32::from_rgb(60, 170, 90), "+")
    } else {
        (egui::Color32::from_rgb(200, 70, 70), "")
    };
    ui.colored_label(color, format!("前ターンの資金変動: {sign}{d} 円"));
}

fn draw_buy(ui: &mut egui::Ui, inv: &Inventory, policy: &mut TradePolicy, ledger: &PlayerLedger) {
    ui.strong("部品購入");
    ui.label(format!("現在の部品在庫: {} 個", inv.parts));
    match ledger.avg_parts_price() {
        Some(p) => ui.label(format!("平均部品調達価格: {p:.1} 円")),
        None => ui.label("平均部品調達価格: — （未取引）"),
    };
    ui.separator();

    ui.add(
        egui::Slider::new(&mut policy.max_buy, UI_MAX_BUY_RANGE.0..=UI_MAX_BUY_RANGE.1)
            .text("最大買付価格"),
    );
    ui.add(
        egui::Slider::new(
            &mut policy.target_stock_turns,
            UI_STOCK_TURNS_RANGE.0..=UI_STOCK_TURNS_RANGE.1,
        )
        .text("目標在庫数（製造設備消費ターン）"),
    );
}

fn draw_sell(ui: &mut egui::Ui, inv: &Inventory, policy: &mut TradePolicy, ledger: &PlayerLedger) {
    ui.strong("製品売却");
    ui.label(format!("現在の自転車在庫: {} 台", inv.bikes));
    match ledger.avg_bike_price() {
        Some(p) => ui.label(format!("平均販売価格: {p:.1} 円")),
        None => ui.label("平均販売価格: — （未取引）"),
    };
    ui.separator();

    ui.add(
        egui::Slider::new(
            &mut policy.min_sell,
            UI_MIN_SELL_RANGE.0..=UI_MIN_SELL_RANGE.1,
        )
        .text("最低売却価格"),
    );
}

fn draw_facility(ui: &mut egui::Ui, fac: &Facility, markets: &Markets) {
    ui.strong("設備");
    egui::Grid::new("facility_grid")
        .num_columns(2)
        .show(ui, |ui| {
            for param in FacilityParam::ALL {
                ui.label(param.label_jp());
                let cur = param.current(fac);
                match param {
                    FacilityParam::Defect => ui.label(format!("{:.2} %", cur * 100.0)),
                    _ => ui.label(format!("{cur:.2} {}", param.unit_jp())),
                };
                ui.end_row();
            }
        });
    ui.separator();

    ui.strong("クイック計算");
    // 最大効率で稼働するとき、毎ターン消費する部品数 = 製造速度 × 資源効率
    let parts_needed = (fac.speed * fac.efficiency).ceil();
    ui.label(format!("最大稼働に必要な部品/ターン: {parts_needed:.0} 個"));

    // 1台あたりコスト = 直近部品約定価格 × 資源効率 + 稼働維持費 + 固定維持費 / 製造速度
    if fac.speed > 0.0 {
        let parts_price = markets.parts.last_price as f32;
        let cost_per_bike =
            parts_price * fac.efficiency + fac.run_cost + fac.fixed_cost / fac.speed;
        ui.label(format!(
            "最大効率時の1台あたりコスト: {cost_per_bike:.1} 円"
        ));
    }
}

fn draw_upgrade(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    draft: &mut UpgradeDraft,
    entity: Entity,
    fac: &Facility,
    baseline: &FacilityBaseline,
    upgrade: Option<&Upgrade>,
) {
    ui.strong("改良プロジェクト");
    match upgrade {
        Some(up) => {
            ui.label(format!(
                "対象: {}（{} / {} ターン）",
                up.target.label_jp(),
                up.elapsed,
                up.total_turns
            ));
            ui.add(
                egui::ProgressBar::new(up.elapsed as f32 / up.total_turns as f32).show_percentage(),
            );
            ui.label(format!("費用: {} 円/ターン", up.cost_per_turn));
            let improved = up.target.improved(fac, &baseline.0);
            let next_e = expected_delta(
                up.target.base_k(),
                up.cost_per_turn as f32,
                improved,
                up.elapsed as f32 + 0.5,
                up.total_turns as f32,
            );
            ui.label(format!("実績Δ累計: {:.4}", up.accumulated));
            ui.label(format!("次ターンの見込みΔ: {next_e:.4}"));
            if ui.button("中断（支払済費用は返らない）").clicked() {
                commands.entity(entity).remove::<Upgrade>();
            }
        }
        None => {
            egui::ComboBox::from_label("改善する要素")
                .selected_text(draft.param.label_jp())
                .show_ui(ui, |ui| {
                    for param in FacilityParam::ALL {
                        ui.selectable_value(&mut draft.param, param, param.label_jp());
                    }
                });
            ui.add(
                egui::Slider::new(
                    &mut draft.cost,
                    UI_UPGRADE_COST_RANGE.0..=UI_UPGRADE_COST_RANGE.1,
                )
                .text("費用/ターン"),
            );
            ui.add(
                egui::Slider::new(
                    &mut draft.turns,
                    UI_UPGRADE_TURNS_RANGE.0..=UI_UPGRADE_TURNS_RANGE.1,
                )
                .text("ターン数"),
            );
            let improved = draft.param.improved(fac, &baseline.0);
            let e_total = expected_total(
                draft.param.base_k(),
                draft.cost as f32,
                improved,
                draft.turns,
            );
            ui.label(format!("見込みΔ合計 E[Δ]: {e_total:.4}"));
            ui.label(format!(
                "総費用: {} 円",
                draft.cost * i64::from(draft.turns)
            ));
            if ui.button("開始").clicked() {
                commands.entity(entity).insert(Upgrade {
                    target: draft.param,
                    cost_per_turn: draft.cost,
                    total_turns: draft.turns,
                    elapsed: 0,
                    accumulated: 0.0,
                });
            }
        }
    }
}
