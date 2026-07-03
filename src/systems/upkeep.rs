//! フェーズ6: 維持費徴収。固定維持費はここで、稼働維持費は生産時に徴収済み

use bevy::prelude::*;

use crate::model::components::*;

pub fn charge_upkeep(mut q: Query<(&mut Wallet, &Facility), Without<Bankrupt>>) {
    for (mut wallet, fac) in &mut q {
        wallet.0 -= fac.fixed_cost.round() as i64;
    }
}
