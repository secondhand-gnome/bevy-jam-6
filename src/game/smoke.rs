use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::game::despawn::DespawnOnRestart;
use crate::game::lifespan::LifespanTimer;
use avian2d::prelude::{LinearVelocity, RigidBody};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use rand::prelude::SliceRandom;

const SMOKE_LIFT_SPEED: f32 = 15.;
const SMOKE_LIFESPAN_S: f32 = 0.5;
const SMOKE_SCALE: f32 = 0.2;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Smoke>();
    app.add_event::<SpawnSmokeEvent>();
    app.register_type::<SmokeAssets>();
    app.load_resource::<SmokeAssets>();

    app.add_systems(
        Update,
        spawn_smoke
            .run_if(resource_exists::<SmokeAssets>)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Smoke;

#[derive(Event, Debug, Default)]
pub struct SpawnSmokeEvent(pub Vec3);

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct SmokeAssets {
    #[dependency]
    smoke_particles: Vec<Handle<Image>>,
}

impl FromWorld for SmokeAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            smoke_particles: vec![
                "images/particles/smoke/smoke_06.png",
                "images/particles/smoke/smoke_07.png",
                "images/particles/smoke/smoke_08.png",
            ]
            .into_iter()
            .map(|path| {
                assets.load_with_settings(path, |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                })
            })
            .collect::<Vec<Handle<Image>>>(),
        }
    }
}

fn smoke(transform: Transform, smoke_assets: &SmokeAssets) -> impl Bundle {
    (
        Name::new("Smoke"),
        DespawnOnRestart,
        RigidBody::Kinematic,
        LifespanTimer(Timer::from_seconds(SMOKE_LIFESPAN_S, TimerMode::Once)),
        LinearVelocity(SMOKE_LIFT_SPEED * Vec2::Y),
        transform.with_scale(Vec3::splat(SMOKE_SCALE)),
        Sprite {
            image: random_smoke_particle(smoke_assets),
            ..default()
        },
    )
}

fn random_smoke_particle(smoke_assets: &SmokeAssets) -> Handle<Image> {
    let rng = &mut rand::thread_rng();
    smoke_assets.smoke_particles.choose(rng).unwrap().clone()
}

fn spawn_smoke(
    mut commands: Commands,
    mut spawn_smoke_events: EventReader<SpawnSmokeEvent>,
    smoke_assets: Res<SmokeAssets>,
) {
    for ev in spawn_smoke_events.read() {
        let transform = Transform::from_translation(ev.0);
        commands.spawn(smoke(transform, &smoke_assets));
    }
}
