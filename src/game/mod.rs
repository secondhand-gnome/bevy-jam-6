mod barn;
mod farm;
pub mod level;
mod plant;
pub mod player;

use crate::game::player::PlayerClickEvent;
use bevy::prelude::*;
use crate::asset_tracking::LoadResource;
use crate::game::plant::{PlantAssets, SowPlantEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerClickEvent>();
    
    app.add_plugins((
        plant::plugin,
        farm::plugin,
        level::plugin,
        player::plugin,
        barn::plugin,
    ));
}
