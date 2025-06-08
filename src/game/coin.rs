use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::game::despawn::DespawnOnRestart;
use crate::game::lifespan::LifespanTimer;
use avian2d::prelude::{LinearVelocity, RigidBody};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

const COIN_LIFT_SPEED: f32 = 25.;
const COIN_LIFESPAN_S: f32 = 0.75;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Coin>();
    app.add_event::<GetCoinEvent>();
    app.register_type::<CoinAssets>();
    app.load_resource::<CoinAssets>();

    app.add_systems(
        Update,
        make_coins
            .run_if(resource_exists::<CoinAssets>)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Coin;

#[derive(Event, Debug, Default)]
pub struct GetCoinEvent(pub Vec3);

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct CoinAssets {
    #[dependency]
    coin_sprite: Handle<Image>,
    #[dependency]
    coin_sound: Handle<AudioSource>,
}

impl FromWorld for CoinAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            coin_sprite: assets.load_with_settings(
                "images/coin.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            coin_sound: assets.load("audio/sound_effects/get_coin.ogg"),
        }
    }
}

fn coin(transform: Transform, coin_assets: &CoinAssets) -> impl Bundle {
    (
        Name::new("Coin"),
        DespawnOnRestart,
        RigidBody::Kinematic,
        LifespanTimer(Timer::from_seconds(COIN_LIFESPAN_S, TimerMode::Once)),
        LinearVelocity(COIN_LIFT_SPEED * Vec2::new(0., 1.)),
        sound_effect(coin_assets.coin_sound.clone()),
        transform,
        Sprite {
            image: coin_assets.coin_sprite.clone(),
            ..default()
        },
    )
}

fn make_coins(
    mut commands: Commands,
    mut get_coin_events: EventReader<GetCoinEvent>,
    coin_assets: Res<CoinAssets>,
) {
    for ev in get_coin_events.read() {
        let transform = Transform::from_translation(ev.0);
        commands.spawn(coin(transform, &coin_assets));
    }
}
