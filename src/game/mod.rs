mod barn;
mod farm;
pub mod level;
mod plant;
pub mod player;

use crate::game::player::PlayerClickEvent;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerClickEvent>();
    app.add_plugins((
        farm::plugin,
        level::plugin,
        plant::plugin,
        player::plugin,
        barn::plugin,
    ));
}
