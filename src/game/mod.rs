mod barn;
mod farm;
pub mod level;
mod plant;
pub mod player;
mod enemy;

use crate::game::player::{PlayerClickEvent, ThrowSeedEvent};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerClickEvent>();
    app.add_event::<ThrowSeedEvent>();

    app.add_plugins((
        plant::plugin,
        enemy::plugin,
        farm::plugin,
        level::plugin,
        player::plugin,
        barn::plugin,
    ));
}
