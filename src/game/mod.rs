pub mod level;
pub mod player;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        // crate::demo::animation::plugin,
        level::plugin,
        // crate::demo::movement::plugin,
        player::plugin,
    ));
}
