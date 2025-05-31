use crate::asset_tracking::LoadResource;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Barn>();

    app.register_type::<BarnAssets>();
    app.load_resource::<BarnAssets>();
}

/// The barn.
pub fn barn(barn_assets: &BarnAssets) -> impl Bundle {
    (
        Name::new("Barn"),
        Barn,
        Sprite {
            image: barn_assets.house.clone(),
            ..default()
        },
        Transform::from_translation(Vec3::new(-350.0, 30.0, 0.0)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Barn;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct BarnAssets {
    #[dependency]
    house: Handle<Image>,
}

impl FromWorld for BarnAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            house: assets.load_with_settings(
                "images/house.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
