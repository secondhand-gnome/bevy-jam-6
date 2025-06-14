//! Spawn the main level.

use crate::game::barn::{BarnAssets, barn};
use crate::game::farm::{FarmAssets, farm};
use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::player::{PlayerAssets, player},
    screens::Screen,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/banjo_romp.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    farm_assets: Res<FarmAssets>,
    barn_assets: Res<BarnAssets>,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            farm(&farm_assets),
            barn(&barn_assets),
            player(&player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
        ],
    ));
}
