use crate::asset_tracking::LoadResource;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::sprite::SpriteImageMode::Tiled;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Farm>();

    app.register_type::<FarmAssets>();
    app.load_resource::<FarmAssets>();
}

pub fn farm(farm_assets: &FarmAssets) -> impl Bundle {
    (
        Name::new("Farm"),
        Farm,
        Sprite {
            image: farm_assets.grass_a.clone(),
            image_mode: Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 0.1,
            },
            ..default()
        },
        Transform::from_scale(Vec3::new(10., 10., 10.)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Farm;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FarmAssets {
    #[dependency]
    grass_a: Handle<Image>,
    #[dependency]
    grass_b: Handle<Image>,
    #[dependency]
    dirt_a: Handle<Image>,
    #[dependency]
    dirt_b: Handle<Image>,
    // TODO plants
}

impl FromWorld for FarmAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            grass_a: assets.load_with_settings(
                "images/ground/grass_a.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            grass_b: assets.load_with_settings(
                "images/ground/grass_b.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            dirt_a: assets.load_with_settings(
                "images/ground/dirt_a.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            dirt_b: assets.load_with_settings(
                "images/ground/dirt_b.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
