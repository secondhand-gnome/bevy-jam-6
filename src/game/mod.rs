mod barn;
mod enemy;
mod farm;
mod health;
pub mod level;
mod physics;
mod plant;
pub mod player;

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
        barn::plugin,
    ));
}
