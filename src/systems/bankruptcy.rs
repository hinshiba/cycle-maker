//! フェーズ7: 倒産判定と勝敗判定
//! 資金 < 0 の企業を市場から除外（プレイヤーならゲームオーバー）。
//! プレイヤー資金が目標額に到達したら勝利

use bevy::prelude::*;

use crate::constants::GOAL_FUNDS;
use crate::model::components::*;
use crate::model::resources::{GameResult, Outcome, TurnCounter};

pub fn judge(
    mut commands: Commands,
    turn: Res<TurnCounter>,
    mut result: ResMut<GameResult>,
    q: Query<(Entity, &Wallet, Option<&Player>), Without<Bankrupt>>,
) {
    for (entity, wallet, player) in &q {
        if player.is_some() {
            if wallet.0 >= GOAL_FUNDS {
                result
                    .0
                    .get_or_insert(Outcome::Victory { turn: turn.0 + 1 });
            } else if wallet.0 < 0 {
                result
                    .0
                    .get_or_insert(Outcome::Bankrupt { turn: turn.0 + 1 });
            }
        } else if wallet.0 < 0 {
            // NPCは倒産マーカーで全システムから除外（UIには倒産と表示）
            commands.entity(entity).insert(Bankrupt).remove::<Upgrade>();
        }
    }
}
