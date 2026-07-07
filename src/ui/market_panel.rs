//! 中央ペインの描画
//! 2市場の直近約定価格
//! 出来高
//! 価格推移チャート
//! 自社の販売実績

use bevy_egui::egui;

use crate::model::resources::{Markets, PlayerStats, PriceHistory};

/// 部品の系列色（青）
const COLOR_PARTS: egui::Color32 = egui::Color32::from_rgb(0x4C, 0x9A, 0xFF);
/// 自転車の系列色（橙）
const COLOR_BIKES: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x8C, 0x42);

pub fn draw(root: &mut egui::Ui, markets: &Markets, stats: &PlayerStats, history: &PriceHistory) {
    egui::CentralPanel::default().show(root, |ui| {
        ui.heading("市場");
        ui.separator();

        egui::Grid::new("markets_grid")
            .num_columns(3)
            .show(ui, |ui| {
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
        ui.heading("価格推移");
        draw_price_chart(ui, history);

        ui.separator();
        ui.heading("自社の直近ターン実績");
        ui.label(format!("生産: {} 台", stats.produced_bikes));
        ui.label(format!(
            "市場売却: {} 台 / 売上 {} 円",
            stats.sold_bikes, stats.revenue
        ));
    });
}

/// 部品と自転車の直近約定価格を同一グラフに折れ線で描く（egui の Painter で自前描画）
fn draw_price_chart(ui: &mut egui::Ui, history: &PriceHistory) {
    let parts = &history.parts;
    let bikes = &history.bikes;
    let n = parts.len().max(bikes.len());

    // 領域を確保（幅は利用可能分いっぱい、高さは固定）
    let desired = egui::vec2(ui.available_width(), 220.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);

    if n == 0 {
        return;
    }

    // 軸ラベル用の余白を除いた作図領域
    let plot = egui::Rect::from_min_max(
        rect.min + egui::vec2(56.0, 10.0),
        rect.max - egui::vec2(10.0, 22.0),
    );

    // 縦軸レンジ（両系列の最小・最大に 5% の余白）
    let (mut y_min, mut y_max) = parts
        .iter()
        .chain(bikes.iter())
        .fold((i64::MAX, i64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    if y_min == y_max {
        y_min -= 1;
        y_max += 1;
    }
    let span = (y_max - y_min) as f64;
    let y_lo = y_min as f64 - span * 0.05;
    let y_hi = y_max as f64 + span * 0.05;
    let x_max = (n - 1) as f64; // ターン0..n-1

    let to_screen = |x: f64, y: f64| -> egui::Pos2 {
        let tx = if x_max > 0.0 { x / x_max } else { 0.0 };
        let ty = (y - y_lo) / (y_hi - y_lo);
        egui::pos2(
            plot.left() + tx as f32 * plot.width(),
            plot.bottom() - ty as f32 * plot.height(),
        )
    };

    let axis_color = ui.visuals().weak_text_color();
    let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    let font = egui::FontId::proportional(11.0);

    // 横グリッド＋縦軸目盛（下端・中央・上端の3本）
    for f in [0.0_f64, 0.5, 1.0] {
        let price = y_lo + (y_hi - y_lo) * f;
        let p = to_screen(0.0, price);
        painter.line_segment(
            [egui::pos2(plot.left(), p.y), egui::pos2(plot.right(), p.y)],
            egui::Stroke::new(1.0, grid_color),
        );
        painter.text(
            egui::pos2(plot.left() - 4.0, p.y),
            egui::Align2::RIGHT_CENTER,
            format!("{}", price.round() as i64),
            font.clone(),
            axis_color,
        );
    }

    // 折れ線（1点しかない系列は点で表示）
    let plot_series = |series: &[i64], color: egui::Color32| {
        let pts: Vec<egui::Pos2> = series
            .iter()
            .enumerate()
            .map(|(i, &v)| to_screen(i as f64, v as f64))
            .collect();
        match pts.len() {
            0 => {}
            1 => {
                painter.circle_filled(pts[0], 2.5, color);
            }
            _ => {
                painter.add(egui::Shape::line(pts, egui::Stroke::new(1.8, color)));
            }
        }
    };
    plot_series(parts, COLOR_PARTS);
    plot_series(bikes, COLOR_BIKES);

    // 凡例（作図領域の左上に色見本＋ラベル）
    let mut legend = plot.left_top() + egui::vec2(4.0, 4.0);
    for (label, color) in [("部品", COLOR_PARTS), ("自転車", COLOR_BIKES)] {
        painter.rect_filled(
            egui::Rect::from_min_size(legend, egui::vec2(10.0, 10.0)),
            2.0,
            color,
        );
        let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), axis_color);
        let w = galley.size().x;
        painter.galley(legend + egui::vec2(14.0, -1.0), galley, axis_color);
        legend.x += 14.0 + w + 12.0;
    }

    // ホバー: 最寄りターンに縦ガイドと両系列の値を表示
    if let Some(pos) = response.hover_pos() {
        if plot.x_range().contains(pos.x) {
            let tx = ((pos.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
            let idx = (tx as f64 * x_max).round() as usize;
            let gx = to_screen(idx as f64, 0.0).x;
            painter.line_segment(
                [egui::pos2(gx, plot.top()), egui::pos2(gx, plot.bottom())],
                egui::Stroke::new(1.0, axis_color),
            );
            let mut lines = vec![format!("ターン {idx}")];
            if let Some(&p) = parts.get(idx) {
                painter.circle_filled(to_screen(idx as f64, p as f64), 3.0, COLOR_PARTS);
                lines.push(format!("部品 {p} 円"));
            }
            if let Some(&b) = bikes.get(idx) {
                painter.circle_filled(to_screen(idx as f64, b as f64), 3.0, COLOR_BIKES);
                lines.push(format!("自転車 {b} 円"));
            }
            response.clone().on_hover_text(lines.join("\n"));
        }
    }
}
