//! 上部バー: ターン数、目標資本金への進捗、ターン終了ボタン

use bevy_egui::egui;

use crate::constants::GOAL_FUNDS;

/// ターン終了がクリックされたら true を返す
pub fn draw(root: &mut egui::Ui, turn: u32, player_wallet: Option<i64>, can_end: bool) -> bool {
    let mut clicked = false;
    egui::Panel::top("top_bar")
        .resizable(false)
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("ターン: {turn}"));
                ui.separator();

                if let Some(wallet) = player_wallet {
                    let frac = (wallet as f32 / GOAL_FUNDS as f32).clamp(0.0, 1.0);
                    ui.label("目標資本金");
                    ui.add_sized(
                        [240.0, 18.0],
                        egui::ProgressBar::new(frac).text(format!("{wallet} / {GOAL_FUNDS}")),
                    );
                }
                ui.separator();

                if ui
                    .add_enabled(can_end, egui::Button::new("▶ ターン終了"))
                    .clicked()
                {
                    clicked = true;
                }
            });
        });
    clicked
}
