use bevy::prelude::*;

/// ターン進行の状態機械: 入力待ち ⇄ 解決中（§3）、決着で GameOver
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    AwaitingInput,
    Resolving,
    GameOver,
}
