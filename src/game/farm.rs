use crate::asset_tracking::LoadResource;
use crate::game::plant::SowPlantEvent;
use crate::game::player::PlayerClickEvent;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::sprite::SpriteImageMode::Tiled;
use bevy_vector_shapes::prelude::*;

const TILE_SIZE_PX: f32 = 128.;
const FARM_SIZE_TILES: Vec2 = Vec2::new(10., 8.);
const FARM_SIZE_PX: Vec2 = Vec2::new(
    FARM_SIZE_TILES.x * TILE_SIZE_PX,
    FARM_SIZE_TILES.y * TILE_SIZE_PX,
);

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Farm>();

    app.register_type::<FarmAssets>();
    app.load_resource::<FarmAssets>();
    app.add_systems(Update, draw_outline);
    app.add_systems(Update, on_player_click);
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
        Transform::from_scale(FARM_SIZE_TILES.extend(1.)),
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

fn draw_outline(mut painter: ShapePainter, q_farm: Query<&Farm>) {
    if q_farm.single().is_ok() {
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.rect(FARM_SIZE_PX);
    }
}

fn on_player_click(
    mut click_events: EventReader<PlayerClickEvent>,
    mut sow_events: EventWriter<SowPlantEvent>,
    q_farm: Query<&Farm>,
) {
    if q_farm.single().is_ok() {
        for click_event in click_events.read() {
            // TODO check if we can actually plant here or not
            let position = click_event.0;
            sow_events.write(SowPlantEvent { position });
        }
    }
}
