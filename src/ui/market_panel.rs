//! 中央パネル: 2市場の直近約定価格・出来高と自社の販売実績（チャートは P1）

use bevy_egui::egui;

use crate::model::resources::{Markets, PlayerStats};

pub fn draw(root: &mut egui::Ui, markets: &Markets, stats: &PlayerStats) {
    egui::CentralPanel::default().show(root, |ui| {
        ui.heading("市場");
        ui.separator();

        egui::Grid::new("markets_grid").num_columns(3).show(ui, |ui| {
            ui.strong("市場");
            ui.strong("直近約定価格");
            ui.strong("出来高");
            ui.end_row();

            ui.label("部品市場");
            ui.label(format!("{} 円", markets.parts.last_price));
            ui.label(format!("{}", markets.parts.volume));
            ui.end_row();

            ui.label("自転車市場");
            ui.label(format!("{} 円", markets.bikes.last_price));
            ui.label(format!("{}", markets.bikes.volume));
            ui.end_row();
        });

        ui.separator();
        ui.heading("自社の直近ターン実績");
        ui.label(format!("生産: {} 台", stats.produced_bikes));
        ui.label(format!(
            "市場売却: {} 台 / 売上 {} 円",
            stats.sold_bikes, stats.revenue
        ));
    });
}
