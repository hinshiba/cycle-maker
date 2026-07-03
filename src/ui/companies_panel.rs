//! 右パネル: 全企業の概況（業種・資金・在庫・倒産表示）

use bevy_egui::egui;

use super::CompaniesQuery;

pub fn draw(root: &mut egui::Ui, companies: &CompaniesQuery) {
    let mut rows: Vec<_> = companies.iter().collect();
    rows.sort_by_key(|(name, kind, ..)| (kind.label_jp(), name.0.clone()));

    egui::Panel::right("companies_panel")
        .default_size(330.0)
        .show(root, |ui| {
            ui.heading("企業一覧");
            ui.separator();
            egui::Grid::new("companies_grid")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("企業");
                    ui.strong("業種");
                    ui.strong("資金");
                    ui.strong("在庫(部/車)");
                    ui.strong("状態");
                    ui.end_row();

                    for (name, kind, wallet, inv, bankrupt, upgrade) in rows {
                        ui.label(&name.0);
                        ui.label(kind.label_jp());
                        ui.label(format!("{}", wallet.0));
                        ui.label(format!("{} / {}", inv.parts, inv.bikes));
                        if bankrupt.is_some() {
                            ui.colored_label(egui::Color32::RED, "倒産");
                        } else if upgrade.is_some() {
                            ui.label("改良中");
                        } else {
                            ui.label("-");
                        }
                        ui.end_row();
                    }
                });
        });
}
