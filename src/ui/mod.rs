//! egui による UI（§10）
//! 上: ターン/進捗/ターン終了、左: 自社パネル、中央: 市場、右: 企業一覧
//! egui 0.35 のパネルはルート Ui に対して追加するため、1つのシステムで全パネルを描く

pub mod companies_panel;
pub mod market_panel;
pub mod player_panel;
pub mod top_bar;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::model::components::*;
use crate::model::resources::*;
use crate::states::GameState;
use player_panel::UpgradeDraft;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(EguiPrimaryContextPass, (setup_fonts, draw_ui).chain());
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// eguiの標準フォントは日本語グリフを持たないため、Noto Sans JP を登録する
fn setup_fonts(mut contexts: EguiContexts, mut done: Local<bool>) -> Result {
    if *done {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_jp".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansJP.ttf")).into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("noto_jp".to_owned());
    }
    ctx.set_fonts(fonts);
    *done = true;
    Ok(())
}

pub type PlayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Wallet,
        &'static Inventory,
        &'static Facility,
        &'static FacilityBaseline,
        &'static mut TradePolicy,
        Option<&'static Upgrade>,
    ),
    With<Player>,
>;

pub type CompaniesQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static CompanyName,
        &'static CompanyKind,
        &'static Wallet,
        &'static Inventory,
        Option<&'static Bankrupt>,
        Option<&'static Upgrade>,
    ),
>;

#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut draft: Local<UpgradeDraft>,
    turn: Res<TurnCounter>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    result: Res<GameResult>,
    markets: Res<Markets>,
    stats: Res<PlayerStats>,
    mut player: PlayerQuery,
    companies: CompaniesQuery,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // パネルの土台になる画面全体のルート Ui（egui 0.35 のパネルAPI）
    let mut root = egui::Ui::new(
        ctx.clone(),
        "root_ui".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    let player_wallet = player.single().map(|(_, w, ..)| w.0).ok();
    let can_end = *state.get() == GameState::AwaitingInput && result.0.is_none();
    if top_bar::draw(&mut root, turn.0, player_wallet, can_end) {
        next.set(GameState::Resolving);
    }
    player_panel::draw(&mut root, &mut commands, &mut draft, &mut player);
    companies_panel::draw(&mut root, &companies);
    market_panel::draw(&mut root, &markets, &stats); // 中央は最後

    // 勝敗が付いたら中央にオーバーレイ表示
    if let Some(outcome) = result.0 {
        egui::Window::new("ゲーム終了")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| match outcome {
                Outcome::Victory { turn } => {
                    ui.heading("🏆 勝利！");
                    ui.label(format!("目標資本金に到達しました（{turn} ターン）"));
                }
                Outcome::Bankrupt { turn } => {
                    ui.heading("倒産…");
                    ui.label(format!("資金がマイナスになりました（{turn} ターン）"));
                }
            });
    }
    Ok(())
}
