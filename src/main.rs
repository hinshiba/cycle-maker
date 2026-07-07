// releaseビルドのみコンソールを開かなくする
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use cycle_maker::{AudioPlugin, SimulationPlugin, UiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "自転車製造会社経営シミュレータ".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins((SimulationPlugin { seed: 42 }, UiPlugin, AudioPlugin))
        .run();
}
