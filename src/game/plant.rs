use crate::asset_tracking::LoadResource;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

const PLANT_RADIUS: f32 = 30.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Plant>();

    app.register_type::<PlantAssets>();
    app.load_resource::<PlantAssets>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Plant;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlantAssets {
    #[dependency]
    daisy: Handle<Image>,
    #[dependency]
    dragonfruit: Handle<Image>,
    #[dependency]
    pineapple: Handle<Image>,
    #[dependency]
    seedling: Handle<Image>,
}

impl FromWorld for PlantAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            daisy: assets.load_with_settings(
                "images/plants/daisy.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            dragonfruit: assets.load_with_settings(
                "images/plants/dragonfruit.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            pineapple: assets.load_with_settings(
                "images/plants/pineapple.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            seedling: assets.load_with_settings(
                "images/plants/seedling.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
