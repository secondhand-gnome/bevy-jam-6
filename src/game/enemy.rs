//! Enemies eat plants.

use crate::asset_tracking::LoadResource;
use avian2d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use rand::Rng;

const ENEMY_RADIUS: f32 = 30.0;
const SPAWN_INTERVAL_S: f32 = 1.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();

    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();

    app.add_systems(Update, tick_spawn.run_if(resource_exists::<EnemyAssets>));

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
