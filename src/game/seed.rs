use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::game::plant::{PlantType, SowPlantEvent};
use crate::game::player::ThrowSeedEvent;
use avian2d::prelude::{LinearVelocity, RigidBody};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use std::fmt::Debug;

const SEED_Z_LAYER: f32 = 2.0;
const SEED_MOVE_SPEED: f32 = 10.;
const SEED_POINT_EPLISON: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Seed>();

    app.register_type::<SeedAssets>();
    app.load_resource::<SeedAssets>();

    app.add_systems(
        Update,
        (create_seeds, move_seeds)
            .run_if(resource_exists::<SeedAssets>)
            .in_set(PausableSystems),
    );
}

pub fn seed(
    seed_assets: &SeedAssets,
    plant_type: PlantType,
    path: SeedPath,
    origin: Vec3,
) -> impl Bundle {
    (
        Name::new("Seed"),
        Seed,
        plant_type,
        path,
        Transform::from_translation(origin),
        Sprite {
            image: seed_assets.seed.clone(),
            ..default()
        },
        RigidBody::Kinematic,
        LinearVelocity::ZERO,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Seed;

#[derive(Component, Debug, Default)]
pub struct SeedPath {
    path: Vec<IVec2>,
}

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

fn create_seeds(
    mut commands: Commands,
    mut throw_seed_events: EventReader<ThrowSeedEvent>,
    seed_assets: Res<SeedAssets>,
) {
    for ev in throw_seed_events.read() {
        let Some(origin) = ev.path.first() else {
            continue;
        };
        info!("Spawned a seed at {:?}", origin);
        commands.spawn(seed(
            &seed_assets,
            ev.seed_type,
            SeedPath {
                path: ev.path.clone(),
            },
            origin.as_vec2().extend(SEED_Z_LAYER),
        ));
    }
}

fn move_seeds(
    mut commands: Commands,
    mut q_seeds: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            &mut SeedPath,
            &PlantType,
        ),
        With<Seed>,
    >,
    mut sow_plants_events: EventWriter<SowPlantEvent>,
    mut throw_seed_events: EventWriter<ThrowSeedEvent>,
) {
    for (seed, seed_transform, mut vel, mut seed_path, seed_type) in q_seeds.iter_mut() {
        if seed_path.path.len() == 0 {
            sow_plants_events.write(SowPlantEvent {
                position: seed_transform.translation.xy(),
                seed_type: *seed_type,
            });
            commands.entity(seed).despawn();

            continue;
        }

        // TODO debug this
        let target = seed_path.path.first().unwrap().as_vec2();
        let vec_to_target = target - seed_transform.translation.xy();
        let dist_from_target = vec_to_target.length();
        if dist_from_target < SEED_POINT_EPLISON {
            // TODO if we reach this point, throw the seed to the next point
            // TODO despawn this seed as well
            commands.entity(seed).despawn();
            throw_seed_events.write(ThrowSeedEvent {
                from_player: false,
                path: seed_path.path[1..].to_owned(),
                seed_type: *seed_type,
            });
        }
        // if seed_transform.translation
        //     * vel = LinearVelocity(
        //     SEED_MOVE_SPEED
        //         * (current_target - seed_transform.translation.xy()).normalize(),
        // );
    }
}
