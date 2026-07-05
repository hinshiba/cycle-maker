//! 倒産判定と勝敗判定
//! 資金 < 0 の企業を市場から除外(プレイヤーならゲームオーバー)
//! 資金が目標額に到達したらその企業の勝利
//! NPC企業が買った場合はゲームオーバー

use bevy::prelude::*;

use crate::constants::GOAL_FUNDS;
use crate::model::components::*;
use crate::model::resources::{GameResult, Outcome, TurnCounter};

/// プレイヤーが同着でも勝利するために先に実行すること
pub fn player_judge(
    turn: Res<TurnCounter>,
    mut result: ResMut<GameResult>,
    q: Query<(Entity, &Wallet), With<Player>>,
) {
    let (_entity, wallet) = q.single().expect("プレイヤー企業が1つではありません");

    if GOAL_FUNDS <= wallet.0 {
        result
            .0
            .get_or_insert(Outcome::Victory { turn: turn.0 + 1 });
        return;
    };

    if wallet.0 < 0 {
        result
            .0
            .get_or_insert(Outcome::Bankrupt { turn: turn.0 + 1 });
        return;
    }
}

/// NPCの勝利判定と破産判定
pub fn npc_judge(
    mut commands: Commands,
    turn: Res<TurnCounter>,
    mut result: ResMut<GameResult>,
    q: Query<(Entity, &Wallet, &CompanyKind), Or<(Without<Bankrupt>, Without<Player>)>>,
) {
    for (entity, wallet, kind) in &q {
        if GOAL_FUNDS <= wallet.0 && kind == &CompanyKind::Manufacturer {
            result
                .0
                .get_or_insert(Outcome::RivalVictory { turn: turn.0 + 1 });
            return;
        };

        if wallet.0 < 0 {
            // NPCは倒産マーカーで全システムから除外
            commands.entity(entity).insert(Bankrupt).remove::<Upgrade>();
        }
    }
}
