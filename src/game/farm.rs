use crate::asset_tracking::LoadResource;
use crate::game::enemy::enemy_spawner;
use crate::game::plant::{Plant, SeedSelection, SowPlantEvent, plant_collision_check};
use crate::game::player::{Player, PlayerClickEvent, ThrowSeedEvent, can_player_reach};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy::sprite::SpriteImageMode::Tiled;
use bevy_cobweb::prelude::Reactive;
use bevy_vector_shapes::prelude::*;

const TILE_SIZE_PX: f32 = 128.;
const FARM_SIZE_TILES: Vec2 = Vec2::new(10., 8.);
const FARM_SIZE_PX: Vec2 = Vec2::new(
    FARM_SIZE_TILES.x * TILE_SIZE_PX,
    FARM_SIZE_TILES.y * TILE_SIZE_PX,
);

const STARTING_BALANCE: f32 = 10.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Farm>();
    app.add_event::<BankAccountUpdateEvent>();

    app.register_type::<FarmAssets>();
    app.load_resource::<FarmAssets>();
    app.add_systems(Update, draw_outline);
    app.add_systems(Update, on_player_click);
}

pub fn farm(farm_assets: &FarmAssets) -> impl Bundle {
    (
        Name::new("Farm"),
        Farm,
        BankAccount {
            balance: STARTING_BALANCE,
        },
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
        children![enemy_spawner(
            Transform::from_translation(Vec3::new(FARM_SIZE_PX.x * 0.5, 0., 0.)),
            FARM_SIZE_PX.y * 0.9
        )],
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

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub struct BankAccount {
    balance: f32,
}

#[derive(Event, Debug, Default)]
pub struct BankAccountUpdateEvent;

impl BankAccount {
    pub fn balance(&self) -> f32 {
        self.balance
    }

    fn deduct(&mut self, amount: f32) {
        self.balance -= amount;
    }
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
        painter.transform = Transform::default();
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.rect(FARM_SIZE_PX);
    }
}

fn on_player_click(
    mut click_events: EventReader<PlayerClickEvent>,
    mut sow_events: EventWriter<SowPlantEvent>,
    mut throw_seed_events: EventWriter<ThrowSeedEvent>,
    q_player: Query<&Transform, With<Player>>,
    q_seed_selection: Reactive<SeedSelection>,
    q_farm: Query<&Farm>,
    q_plants: Query<&Transform, With<Plant>>,
    mut q_bank_account: Query<&mut BankAccount>,
    mut bank_account_update_events: EventWriter<BankAccountUpdateEvent>,
) {
    if q_farm.single().is_ok() {
        for click_event in click_events.read() {
            let click_position = click_event.0;

            let (_, seed_selection) = q_seed_selection.single();
            let seed_type = seed_selection.seed_type();

            let mut can_sow = true;

            if let Ok(player_transform) = q_player.single() {
                let player_position = player_transform.translation.xy();
                if !can_player_reach(player_position, click_position) {
                    can_sow = false;
                }
                // TODO consider the gnomes
            } else {
                error!("No player found!");
                return;
            }

            for plant_transform in q_plants.iter() {
                let plant_position = plant_transform.translation.xy();
                if plant_collision_check(plant_position, click_position) {
                    // Plant already here
                    println!(
                        "Can't plant at {:?} - plant already present at {:?}",
                        click_position, plant_position
                    );
                    can_sow = false;
                }
            }

            let Ok(mut bank_account) = q_bank_account.single_mut() else {
                warn!("No bank account!");
                return;
            };
            info!(
                "To plant {:?} would cost {}. We have {}",
                seed_type,
                seed_type.price(),
                bank_account.balance()
            );
            if bank_account.balance() < seed_type.price() {
                can_sow = false;
                info!("Can't afford seed");
            }

            if can_sow {
                // Actually sow a plant
                // TODO don't actually sow until seed hits the ground
                sow_events.write(SowPlantEvent {
                    position: click_position,
                    seed_type,
                });

                bank_account.deduct(seed_type.price());
                bank_account_update_events.write(BankAccountUpdateEvent);

                // TODO handle multiple throws in a chain
                throw_seed_events.write(ThrowSeedEvent {
                    origin: click_position,
                });
            }
        }
    }
}
