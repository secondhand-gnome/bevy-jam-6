use crate::asset_tracking::LoadResource;
use crate::game::plant::{PlantType, SowPlantEvent};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Seed>();

    app.register_type::<SeedAssets>();
    app.load_resource::<SeedAssets>();

    // TODO make seeds move and take root
}

pub fn seed(seed_assets: &SeedAssets, plant_type: PlantType) -> impl Bundle {
    (Name::new("Seed"), Seed, plant_type)
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Seed;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SeedAssets {
    #[dependency]
    seed: Handle<Image>,
}

impl FromWorld for SeedAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            seed: assets.load_with_settings(
                "images/seed.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
