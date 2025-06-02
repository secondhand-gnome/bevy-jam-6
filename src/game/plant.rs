use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::game::health::Health;
use crate::game::physics::GameLayer;
use crate::theme::palette::{PLANT_GROWTH_FOREGROUND, PLANT_GROWTH_OUTLINE, PLANT_HEALTH_OUTLINE};
use avian2d::prelude::{Collider, CollisionLayers, RigidBody};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::prelude::*;
use rand::prelude::SliceRandom;

const PLANT_RADIUS_PX: f32 = 30.;
const DAISY_GROWTH_TIME_S: f32 = 3.;
const PLANT_MAX_HEALTH: i32 = 5; // TODO depends on plant type

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Plant>();

    app.register_type::<PlantAssets>();
    app.load_resource::<PlantAssets>();

    app.add_event::<SowPlantEvent>();
    app.add_event::<DamagePlantEvent>();
    app.add_systems(
        Update,
        (sow_plants, damage_plants).run_if(resource_exists::<PlantAssets>),
    );
    app.add_systems(
        Update,
        tick_growth
            .run_if(resource_exists::<PlantAssets>)
            .in_set(PausableSystems),
    );
    app.add_systems(Update, (draw_plant_circles, draw_growth, draw_health));
}

fn plant(position: Vec2, plant_assets: &PlantAssets) -> impl Bundle {
    (
        Name::new(format!("Plant at {:?}", position)),
        Plant,
        RigidBody::Static,
        Collider::circle(PLANT_RADIUS_PX),
        CollisionLayers::new([GameLayer::Plant], [GameLayer::Plant, GameLayer::Enemy]),
        Sprite {
            image: plant_assets.seedling.clone(),
            ..default()
        },
        GrowthTimer(Timer::from_seconds(DAISY_GROWTH_TIME_S, TimerMode::Once)),
        Health::new(PLANT_MAX_HEALTH),
        Transform::from_translation(position.extend(1.)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Plant; // TODO require a plant type

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct GrowthTimer(Timer);

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
    #[dependency]
    sow_sounds: Vec<Handle<AudioSource>>,
    #[dependency]
    growth_sound: Handle<AudioSource>,
    #[dependency]
    death_sound: Handle<AudioSource>,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum PlantType {
    #[default]
    Daisy,
    Pineapple,
    Dragonfruit,
}

#[derive(ReactComponent, Default, Clone, Copy)]
pub struct SeedSelection {
    seed_type: PlantType,
}

impl SeedSelection {
    pub fn set_seed_type(&mut self, seed_type: PlantType) {
        info!("Set seed type to {:?}", seed_type);
        self.seed_type = seed_type;
    }

    pub fn seed_type(&self) -> PlantType {
        self.seed_type
    }
}

#[derive(Event, Debug, Default)]
pub struct SowPlantEvent {
    pub position: Vec2,
    pub seed_type: PlantType,
}

// TODO generalize for enemies as well
#[derive(Event, Debug)]
pub struct DamagePlantEvent {
    pub plant_entity: Entity,
    pub amount: i32,
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
            sow_sounds: vec![
                assets.load("audio/sound_effects/sow1.ogg"),
                assets.load("audio/sound_effects/sow2.ogg"),
            ],
            growth_sound: assets.load("audio/sound_effects/growth.ogg"),
            death_sound: assets.load("audio/sound_effects/death.ogg"),
        }
    }
}

fn sow_plants(
    mut commands: Commands,
    plant_assets: Res<PlantAssets>,
    mut sow_events: EventReader<SowPlantEvent>,
) {
    for event in sow_events.read() {
        println!(
            "Plant ({:?}) spawned at {:?}",
            event.seed_type, event.position
        );
        commands.spawn(plant(event.position, &plant_assets));

        let rng = &mut rand::thread_rng();
        let random_sow_sound = plant_assets.sow_sounds.choose(rng).unwrap().clone();
        commands.spawn((
            sound_effect(random_sow_sound),
            Transform::from_translation(event.position.extend(0.)),
        ));
    }
}

fn draw_plant_circles(mut painter: ShapePainter, q_plants: Query<&Transform, With<Plant>>) {
    painter.color = PLANT_GROWTH_OUTLINE;
    painter.hollow = true;
    painter.thickness = 0.5;
    for plant_transform in q_plants {
        painter.transform.translation = plant_transform.translation;
        painter.circle(PLANT_RADIUS_PX);
    }
}

fn draw_growth(
    mut painter: ShapePainter,
    q_growing_plants: Query<(&mut Transform, &mut GrowthTimer), With<Plant>>,
) {
    const PROGRESS_HEIGHT_PX: f32 = PLANT_RADIUS_PX * 0.2;
    const PROGRESS_LENGTH_PX: f32 = PLANT_RADIUS_PX * 1.;
    const PROGRESS_DIMENS: Vec2 = Vec2::new(PROGRESS_LENGTH_PX, PROGRESS_HEIGHT_PX);
    const PROGRESS_OFFSET: Vec3 = Vec3::new(0., -1.1 * PLANT_RADIUS_PX, 0.);

    for (transform, growth_timer) in q_growing_plants {
        // Draw the remaining time
        painter.transform.translation = transform.translation + PROGRESS_OFFSET;
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.color = PLANT_GROWTH_OUTLINE;
        painter.rect(PROGRESS_DIMENS);

        let progress = growth_timer.0.fraction();
        painter.hollow = false;
        painter.color = PLANT_GROWTH_FOREGROUND;
        painter.rect(Vec2::new(
            PROGRESS_DIMENS.x * progress,
            PROGRESS_DIMENS.y * 0.8,
        ));
    }
}

fn draw_health(mut painter: ShapePainter, q_plants: Query<(&Transform, &Health), With<Plant>>) {
    const HEALTH_HEIGHT_PX: f32 = PLANT_RADIUS_PX * 0.2;
    const HEALTH_LENGTH_PX: f32 = PLANT_RADIUS_PX * 1.;
    const HEALTH_DIMENS: Vec2 = Vec2::new(HEALTH_LENGTH_PX, HEALTH_HEIGHT_PX);
    const HEALTH_OFFSET: Vec3 = Vec3::new(0., 1.1 * PLANT_RADIUS_PX, 0.);

    for (transform, health) in q_plants {
        // Draw the remaining health
        painter.transform.translation = transform.translation + HEALTH_OFFSET;
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.color = PLANT_HEALTH_OUTLINE;
        painter.rect(HEALTH_DIMENS);

        painter.hollow = false;
        painter.color = health.bar_color();
        painter.rect(Vec2::new(
            HEALTH_DIMENS.x * health.fraction(),
            HEALTH_DIMENS.y * 0.8,
        ));
    }
}

fn tick_growth(
    mut commands: Commands,
    mut q_growing_plants: Query<(Entity, &mut Transform, &mut GrowthTimer)>,
    time: Res<Time>,
    plant_assets: Res<PlantAssets>,
) {
    for (entity, mut transform, mut growth_timer) in &mut q_growing_plants {
        growth_timer.0.tick(time.delta());
        if growth_timer.0.finished() {
            commands
                .entity(entity)
                .remove::<GrowthTimer>()
                .remove::<Sprite>()
                .insert(Sprite {
                    // TODO check plant type
                    image: plant_assets.daisy.clone(),
                    ..default()
                });

            // TODO check plant type
            transform.scale = Vec3::splat(0.5);

            commands.spawn((
                sound_effect(plant_assets.growth_sound.clone()),
                Transform::from_translation(transform.translation),
            ));

            println!("Plant {:?} finished growing", entity);
        }
    }
}

fn damage_plants(
    mut q_plants: Query<(Entity, &mut Health), With<Plant>>, // TODO plant health
    mut damage_plant_events: EventReader<DamagePlantEvent>,
) {
    // TODO particle effects on plant damage
    for ev in damage_plant_events.read() {
        for (plant_entity, mut plant_health) in q_plants.iter_mut() {
            if plant_entity == ev.plant_entity {
                plant_health.reduce(ev.amount);
                info!("Damage plant {:?} for {}", plant_entity, ev.amount);
            }
        }
    }
}
