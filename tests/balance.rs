//! ヘッドレスで100ターン自動実行するバランステスト（仕様 §14）
//! UIなしの MinimalPlugins 構成でシミュレーションだけを回し、経済が破綻しないことを確認する

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use cycle_maker::model::components::*;
use cycle_maker::model::resources::*;
use cycle_maker::states::GameState;
use cycle_maker::SimulationPlugin;

fn headless_app(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(SimulationPlugin { seed });
    // Startup（企業スポーン）を走らせる
    app.update();
    // プレイヤーも AiTraits を付けて NPC と同じルールで動かす
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(player).insert(AiTraits {
        aggressiveness: 1.0,
        invest_bias: 1.0,
        margin: 0.2,
    });
    app
}

/// 1ターン進める（Resolving への遷移 → 解決 → AwaitingInput へ復帰）
fn step_turn(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Resolving);
    app.update(); // 遷移が適用され OnEnter(Resolving) のチェーンが走る
    app.update(); // AwaitingInput へ戻る遷移を適用
}

#[test]
fn hundred_turns_economy_survives() {
    let mut app = headless_app(7);

    for _ in 0..100 {
        if app.world().resource::<GameResult>().0.is_some() {
            break;
        }
        step_turn(&mut app);
    }

    let turns = app.world().resource::<TurnCounter>().0;
    assert!(turns >= 30, "経済が早期に決着/崩壊: {turns} ターンで終了");

    // 市場価格は正の値を維持する
    let markets = app.world().resource::<Markets>();
    assert!(markets.parts.last_price > 0, "部品価格が非正");
    assert!(markets.bikes.last_price > 0, "自転車価格が非正");

    // 安全弁: 販売会社が全滅しない（連鎖崩壊防止、仕様 §5・§14）
    let mut q = app.world_mut().query::<(&CompanyKind, Option<&Bankrupt>)>();
    let alive_retailers = q
        .iter(app.world())
        .filter(|(kind, bankrupt)| **kind == CompanyKind::Retailer && bankrupt.is_none())
        .count();
    assert!(alive_retailers > 0, "販売会社が全滅した");

    // 資金が発散していない
    let mut wallets = app.world_mut().query::<&Wallet>();
    for w in wallets.iter(app.world()) {
        assert!(w.0.abs() < 100_000_000, "資金が発散: {}", w.0);
    }
}

#[test]
fn reaching_goal_wins() {
    use cycle_maker::constants::GOAL_FUNDS;
    let mut app = headless_app(1);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world())
        .unwrap();
    // ターン内の支出（部品買付・維持費）があっても判定時に目標を上回る額にする
    app.world_mut().entity_mut(player).get_mut::<Wallet>().unwrap().0 = GOAL_FUNDS + 100_000;
    step_turn(&mut app);
    assert!(
        matches!(
            app.world().resource::<GameResult>().0,
            Some(Outcome::Victory { .. })
        ),
        "目標資本金到達で勝利にならない"
    );
}

#[test]
fn player_bankruptcy_loses() {
    let mut app = headless_app(1);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(player).get_mut::<Wallet>().unwrap().0 = -10_000;
    step_turn(&mut app);
    assert!(
        matches!(
            app.world().resource::<GameResult>().0,
            Some(Outcome::Bankrupt { .. })
        ),
        "資金マイナスで敗北にならない"
    );
}

#[test]
fn same_seed_is_deterministic() {
    let run = |seed: u64| {
        let mut app = headless_app(seed);
        for _ in 0..30 {
            step_turn(&mut app);
        }
        let m = app.world().resource::<Markets>();
        (m.parts.last_price, m.bikes.last_price, m.bikes.volume)
    };
    assert_eq!(run(42), run(42), "同一シードで結果が再現しない");
}
