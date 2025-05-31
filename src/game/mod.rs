pub mod level;
pub mod player;
mod barn;
mod farm;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        farm::plugin,
        level::plugin,
        player::plugin,
        barn::plugin,
    ));
}
