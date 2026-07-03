//! 左パネル: 自社（プレイヤー製造会社）の資金・在庫・設備値・売買方針・改良プロジェクト

use bevy::prelude::*;
use bevy_egui::egui;

use super::PlayerQuery;
use crate::constants::*;
use crate::core::upgrade::{expected_delta, expected_total};
use crate::model::components::*;

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
) {
    let Ok((entity, wallet, inv, fac, baseline, mut policy, upgrade)) = player.single_mut() else {
        return;
    };

    egui::Panel::left("player_panel")
        .default_size(310.0)
        .show(root, |ui| {
            ui.heading("自社（自転車製造）");
            ui.separator();

            ui.label(format!("資金: {} 円", wallet.0));
            ui.label(format!("在庫: 部品 {} / 自転車 {}", inv.parts, inv.bikes));
            ui.separator();

            ui.strong("設備");
            egui::Grid::new("facility_grid").num_columns(2).show(ui, |ui| {
                for param in FacilityParam::ALL {
                    ui.label(param.label_jp());
                    let cur = param.current(fac);
                    let base = param.current(&baseline.0);
                    ui.label(format!("{cur:.3}（初期 {base:.3}）"));
                    ui.end_row();
                }
            });
            ui.separator();

            ui.strong("売買方針");
            ui.add(
                egui::Slider::new(&mut policy.max_buy, UI_MAX_BUY_RANGE.0..=UI_MAX_BUY_RANGE.1)
                    .text("部品の最大買付価格"),
            );
            ui.add(
                egui::Slider::new(&mut policy.min_sell, UI_MIN_SELL_RANGE.0..=UI_MIN_SELL_RANGE.1)
                    .text("自転車の最小販売価格"),
            );
            ui.add(
                egui::Slider::new(
                    &mut policy.target_stock_turns,
                    UI_STOCK_TURNS_RANGE.0..=UI_STOCK_TURNS_RANGE.1,
                )
                .text("目標在庫ターン数"),
            );
            ui.separator();

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
                        egui::ProgressBar::new(up.elapsed as f32 / up.total_turns as f32)
                            .show_percentage(),
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
                    ui.label(format!("総費用: {} 円", draft.cost * i64::from(draft.turns)));
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
        });
}
