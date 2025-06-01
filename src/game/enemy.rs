//! Enemies eat plants.

use crate::asset_tracking::LoadResource;
use crate::game::physics::GameLayer;
use crate::game::plant::Plant;
use avian2d::prelude::Collider;
use avian2d::prelude::CollisionLayers;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::RigidBody;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use rand::Rng;

const ENEMY_RADIUS: f32 = 30.0;
const SPAWN_INTERVAL_S: f32 = 1.0;
const ENEMY_MOVE_SPEED: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();

    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();

    app.add_systems(Update, tick_spawn.run_if(resource_exists::<EnemyAssets>));
    app.add_systems(Update, pursue_plants);

    // TODO for enemies, find nearest plant using physics / colliders.
    // Then walk towards it until close enough to eat it.
}

pub fn enemy_spawner(transform: Transform, spawn_height: f32) -> impl Bundle {
    (
        Name::new("Enemy Spawner"),
        EnemySpawner { spawn_height },
        transform,
        SpawnTimer(Timer::from_seconds(SPAWN_INTERVAL_S, TimerMode::Repeating)),
    )
}

fn enemy(spawn_position: Vec3, enemy_assets: &EnemyAssets) -> impl Bundle {
    (
        Name::new("Enemy"),
        Enemy,
        RigidBody::Kinematic,
        Collider::circle(ENEMY_RADIUS),
        CollisionLayers::new([GameLayer::Enemy], [GameLayer::Plant, GameLayer::Enemy]),
        LinearVelocity(Vec2::new(-10.0, 0.)), // TODO not this
        Sprite {
            image: enemy_assets.rat.clone(),
            ..default()
        },
        Transform::from_translation(spawn_position),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Enemy;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EnemyAssets {
    #[dependency]
    rat: Handle<Image>,
    rat_dead: Handle<Image>,
    rat_hit: Handle<Image>,
    rat_walk: Handle<Image>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct EnemySpawner {
    spawn_height: f32,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct SpawnTimer(Timer);

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            rat: assets.load_with_settings(
                "images/enemies/rat/rat.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            rat_dead: assets.load_with_settings(
                "images/enemies/rat/rat_dead.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            rat_hit: assets.load_with_settings(
                "images/enemies/rat/rat_hit.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            rat_walk: assets.load_with_settings(
                "images/enemies/rat/rat_walk.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

fn tick_spawn(
    mut commands: Commands,
    q_enemy_spawners: Query<(&Transform, &mut SpawnTimer, &EnemySpawner)>,
    time: Res<Time>,
    enemy_assets: Res<EnemyAssets>,
) {
    for (transform, mut spawn_timer, enemy_spawner) in q_enemy_spawners {
        spawn_timer.0.tick(time.delta());

        if spawn_timer.0.just_finished() {
            let rng = &mut rand::thread_rng();
            let rand_f32: f32 = rng.r#gen();
            let y_offset = (rand_f32 - 0.5) * enemy_spawner.spawn_height;
            let mut spawn_position = transform.translation;
            spawn_position.y += y_offset;

            println!(
                "Spawning an enemy at {:?}",
                transform.translation + spawn_position
            );
            commands.spawn(enemy(spawn_position, &enemy_assets));
        }
    }
}

fn pursue_plants(
    // commands: Commands,
    mut q_enemies: Query<(&Transform, &mut LinearVelocity), With<Enemy>>,
    q_plants: Query<&Transform, With<Plant>>,
    // spatial_query: SpatialQuery,
) {
    // TODO use navmesh / A* pathfinding instead of a straight line
    for (enemy_transform, mut enemy_velocity) in q_enemies.iter_mut() {
        if q_plants.is_empty() {
            // No plants - hold still
            *enemy_velocity = LinearVelocity(Vec2::ZERO);
            continue;
        }

        let mut vectors_toward_plants: Vec<_> = q_plants
            .iter()
            .map(|plant_transform| plant_transform.translation - enemy_transform.translation)
            .collect();

        // Sort plants by distance from this enemy
        vectors_toward_plants.sort_by(|a, b| a.length().partial_cmp(&b.length()).unwrap());
        let closest_plant_vector = vectors_toward_plants.first().unwrap();
        *enemy_velocity = LinearVelocity(ENEMY_MOVE_SPEED * closest_plant_vector.xy());

        // TODO uncomment for when we want to eat plants
        // const ENEMY_VISION_RADIUS: f32 = 2000.;
        // let intersections = spatial_query.shape_intersections(
        //     &Collider::circle(ENEMY_VISION_RADIUS),
        //     enemy_transform.translation.xy(),
        //     0.,
        //     &SpatialQueryFilter::from_mask(GameLayer::Plant.to_bits()),
        // );
    }
}
