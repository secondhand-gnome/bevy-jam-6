mod barn;
mod despawn;
mod enemy;
mod farm;
mod health;
pub mod level;
mod physics;
mod plant;
pub mod player;
mod player_animation;
mod seed;
pub mod ui;

use crate::game::player::{PlayerClickEvent, ThrowSeedEvent};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerClickEvent>();
    app.add_event::<ThrowSeedEvent>();

    app.add_plugins(PhysicsPlugins::default());

    app.add_plugins((
        health::plugin,
        plant::plugin,
        enemy::plugin,
        farm::plugin,
        level::plugin,
        player::plugin,
        player_animation::plugin,
        seed::plugin,
        ui::plugin,
        barn::plugin,
    ));
}
