use crate::asset_tracking::LoadResource;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::prelude::*;

const PLANT_RADIUS_PX: f32 = 30.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Plant>();

    app.register_type::<PlantAssets>();
    app.load_resource::<PlantAssets>();

    app.add_event::<SowPlantEvent>();
    app.add_systems(Update, sow_plants.run_if(resource_exists::<PlantAssets>));
    app.add_systems(Update, draw_plant_circles);
}

fn plant(position: Vec2, plant_assets: &PlantAssets) -> impl Bundle {
    (
        Name::new(format!("Plant at {:?}", position)),
        Plant,
        Sprite {
            image: plant_assets.seedling.clone(),
            ..default()
        },
        Transform::from_translation(position.extend(1.)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Plant; // TODO require a plant type

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

#[derive(Event, Debug, Default)]
pub struct SowPlantEvent {
    pub position: Vec2,
    // TODO plant type
}

pub fn plant_collision_check(plant_position: Vec2, hit_position: Vec2) -> bool {
    let difference = plant_position - hit_position;
    difference.length() < PLANT_RADIUS_PX * 2.
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

fn sow_plants(
    mut commands: Commands,
    plant_assets: Res<PlantAssets>,
    mut sow_events: EventReader<SowPlantEvent>,
) {
    for event in sow_events.read() {
        println!("Plant spawned at {:?}", event.position);
        commands.spawn(plant(event.position, &plant_assets));
    }
}

fn draw_plant_circles(mut painter: ShapePainter, q_plants: Query<&Transform, With<Plant>>) {
    for plant_pos in q_plants.iter() {
        painter.transform = *plant_pos;
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.circle(PLANT_RADIUS_PX);
    }
}
