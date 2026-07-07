//! フェーズ1: 全社の進行中改良プロジェクトに成果Δを加算し、費用を徴収する

use bevy::prelude::*;
use rand::RngExt;

use crate::constants::*;
use crate::core::upgrade::upgrade_delta;
use crate::model::components::*;
use crate::model::resources::GameRng;

pub fn advance_upgrades(
    mut commands: Commands,
    mut rng: ResMut<GameRng>,
    mut q: Query<
        (
            Entity,
            &mut Wallet,
            &mut Facility,
            &FacilityBaseline,
            &mut Upgrade,
        ),
        Without<Bankrupt>,
    >,
) {
    for (entity, mut wallet, mut facility, baseline, mut up) in &mut q {
        // 費用を払えなければ中断（支払済費用は返らない）
        if wallet.0 < up.cost_per_turn {
            commands.entity(entity).remove::<Upgrade>();
            continue;
        }
        wallet.0 -= up.cost_per_turn;

        let improved = up.target.improved(&facility, &baseline.0);
        let eps = rng.0.random_range(UPGRADE_EPS_MIN..=UPGRADE_EPS_MAX);
        // ターン内の代表点として t+0.5 を使う（端の g=0 で丸ごと無駄になるのを避ける）
        let delta = upgrade_delta(
            up.target.base_k(),
            up.cost_per_turn as f32,
            improved,
            up.elapsed as f32 + 0.5,
            up.total_turns as f32,
            eps,
        );
        up.target.apply(&mut facility, delta);
        up.accumulated += delta;
        up.elapsed += 1;

        if up.elapsed >= up.total_turns {
            commands.entity(entity).remove::<Upgrade>();
        }
    }
}
