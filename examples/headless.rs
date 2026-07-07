//! バランス調整用のヘッドレス実行: 100ターン回して経済指標を10ターンごとに出力する
//! 使い方: cargo run --example headless

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use cycle_maker::SimulationPlugin;
use cycle_maker::model::components::*;
use cycle_maker::model::resources::*;
use cycle_maker::states::GameState;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(SimulationPlugin { seed: 7 });
    app.update();

    // プレイヤーもNPCと同じAIで動かす
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

    println!(
        "turn | parts(価格/出来高) | bikes(価格/出来高) | 企業資金(供給/製造/販売の平均) | 倒産"
    );
    for i in 0..100 {
        if app.world().resource::<GameResult>().0.is_some() {
            break;
        }
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Resolving);
        app.update();
        app.update();

        if (i + 1) % 10 == 0 {
            let world = app.world_mut();
            let mut q = world.query::<(&CompanyKind, &Wallet, Option<&Bankrupt>)>();
            let mut avg = |target: CompanyKind| {
                let v: Vec<i64> = q
                    .iter(world)
                    .filter(|(k, _, b)| **k == target && b.is_none())
                    .map(|(_, w, _)| w.0)
                    .collect();
                if v.is_empty() {
                    0
                } else {
                    v.iter().sum::<i64>() / v.len() as i64
                }
            };
            let (sup, man, ret) = (
                avg(CompanyKind::Supplier),
                avg(CompanyKind::Manufacturer),
                avg(CompanyKind::Retailer),
            );
            let bankrupt = q.iter(world).filter(|(_, _, b)| b.is_some()).count();
            let m = world.resource::<Markets>();
            println!(
                "{:4} | {:5}円 / {:3} | {:5}円 / {:3} | {:8} / {:8} / {:8} | {}",
                i + 1,
                m.parts.last_price,
                m.parts.volume,
                m.bikes.last_price,
                m.bikes.volume,
                sup,
                man,
                ret,
                bankrupt
            );
        }
    }
    if let Some(outcome) = app.world().resource::<GameResult>().0 {
        println!("結果: {outcome:?}");
    }
}
