use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::game::despawn::DespawnOnRestart;
use crate::game::enemy::enemy_spawner;
use crate::game::plant::{
    DAISY_CHAIN_LENGTH, GNOME_THROW_RADIUS_PX, GrowthTimer, Plant, PlantType, SeedSelection,
    plant_collision_check,
};
use crate::game::player::{
    PLAYER_THROW_RADIUS_PX, Player, PlayerClickEvent, ThrowSeedEvent, throw_path,
};
use crate::theme::palette::{GNOME_THROW_OUTLINE, LOSER_BACKGROUND, WINNER_BACKGROUND};
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
pub const WINNING_BALANCE: f32 = 150.0;
const LOSING_BALANCE: f32 = 0.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Farm>();
    app.add_event::<BankAccountUpdateEvent>();
    app.add_event::<RestartGameEvent>();

    app.register_type::<FarmAssets>();
    app.load_resource::<FarmAssets>();
    app.add_systems(Update, draw_outline);
    app.add_systems(
        Update,
        (
            end_game,
            end_game_button_system,
            on_player_click,
            restart_game,
        )
            .run_if(resource_exists::<FarmAssets>)
            .in_set(PausableSystems),
    );
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
    #[dependency]
    chain_cutters: Handle<Image>,
    #[dependency]
    invalid_sound: Handle<AudioSource>,
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Component)]
struct EndGameDisplay;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Component)]
struct EndGameRestartButton;

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

    pub fn credit(&mut self, amount: f32) {
        self.balance += amount;
    }
}

#[derive(Event, Debug, Default)]
pub struct RestartGameEvent;

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
            chain_cutters: assets.load_with_settings(
                "images/chain_cutters.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            invalid_sound: assets.load("audio/sound_effects/invalid.ogg"),
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
    mut commands: Commands,
    mut click_events: EventReader<PlayerClickEvent>,
    mut throw_seed_events: EventWriter<ThrowSeedEvent>,
    q_player: Query<&Transform, With<Player>>,
    q_seed_selection: Reactive<SeedSelection>,
    q_farm: Query<&Farm>,
    q_plants: Query<(&Transform, &Plant)>,
    q_grown_plants: Query<(&Transform, &Plant), Without<GrowthTimer>>,
    mut q_bank_account: Query<&mut BankAccount>,
    mut bank_account_update_events: EventWriter<BankAccountUpdateEvent>,
    farm_assets: Res<FarmAssets>,
) {
    if q_farm.single().is_ok() {
        for click_event in click_events.read() {
            let click_position = click_event.0;

            let (_, seed_selection) = q_seed_selection.single();
            let seed_type = seed_selection.seed_type();

            let mut can_sow = true;

            let Ok(player_transform) = q_player.single() else {
                error!("No player found!");
                return;
            };

            let player_position = player_transform.translation.xy().as_ivec2();
            let gnome_positions: Vec<IVec2> = q_grown_plants
                .iter()
                .filter(|(_, p)| p.plant_type() == PlantType::Gnome)
                .map(|(t, _)| t.translation.xy().as_ivec2())
                .collect();

            let seed_path = throw_path(
                player_position,
                gnome_positions,
                click_position.as_ivec2(),
                PLAYER_THROW_RADIUS_PX,
                GNOME_THROW_RADIUS_PX,
            );

            // TODO Disallow planting within radius of already requested planting destination
            // TODO restrict throwing just to within the farm territory

            if seed_path.is_none() {
                info!(
                    "No path to throw from {:?} to {:?}",
                    player_position, click_position
                );

                // Play sound for invalid location
                commands.spawn((
                    sound_effect(farm_assets.invalid_sound.clone()),
                    Transform::from_translation(click_position.extend(0.)),
                ));
                can_sow = false;
            }

            for (plant_transform, plant) in q_plants.iter() {
                let plant_position = plant_transform.translation.xy();
                if plant_collision_check(plant_position, click_position, plant.plant_type()) {
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
                bank_account.deduct(seed_type.price());
                bank_account_update_events.write(BankAccountUpdateEvent);

                throw_seed_events.write(ThrowSeedEvent {
                    from_player: true,
                    path: seed_path.unwrap(),
                    seed_type,
                });
            }
        }
    }
}

fn restart_game(
    mut commands: Commands,
    mut events: EventReader<RestartGameEvent>,
    mut q_entities: Query<Entity, With<DespawnOnRestart>>,
    mut q_bank_account: Query<&mut BankAccount>,
    mut ev_bank_account_update: EventWriter<BankAccountUpdateEvent>,
) {
    for _ in events.read() {
        info!("Receive restart event");
        q_bank_account.single_mut().unwrap().balance = STARTING_BALANCE;
        ev_bank_account_update.write_default();

        for entity in q_entities.iter_mut() {
            debug!("Despawning {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

fn end_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_plants: Query<&Plant>,
    q_bank_account: Query<&BankAccount>,
    q_despawn_restart: Query<Entity, (With<DespawnOnRestart>, Without<EndGameDisplay>)>,
    farm_assets: Res<FarmAssets>,
) {
    let Ok(bank_account) = q_bank_account.single() else {
        // No bank account
        return;
    };
    if bank_account.balance <= LOSING_BALANCE {
        let daisy_count = q_plants
            .iter()
            .filter(|p| p.plant_type() == PlantType::Daisy)
            .count();
        if daisy_count >= DAISY_CHAIN_LENGTH {
            // There's still a chance
            return;
        }
        commands.spawn(end_game_text(
            Name::new("GameOverText"),
            "You ran out of money",
            LOSER_BACKGROUND,
            asset_server,
            &farm_assets,
        ));
    } else if bank_account.balance >= WINNING_BALANCE {
        commands.spawn(end_game_text(
            Name::new("WinGameText"),
            "You earned enough money to buy chain cutters! You win!",
            WINNER_BACKGROUND,
            asset_server,
            &farm_assets,
        ));
    } else {
        return;
    }

    // Despawn everything and show a restart button
    for entity in q_despawn_restart {
        commands.entity(entity).despawn();
    }
}

fn end_game_text(
    name: Name,
    text: &str,
    background_color: Color,
    asset_server: Res<AssetServer>,
    farm_assets: &FarmAssets,
) -> impl Bundle {
    (
        name,
        EndGameDisplay,
        DespawnOnRestart,
        Node {
            width: Val::Percent(90.0),
            height: Val::Percent(90.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(5.),
            top: Val::Percent(5.),
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(background_color),
        children![
            (
                ImageNode::new(farm_assets.chain_cutters.clone()),
                Node {
                    min_width: Val::Px(100.),
                    min_height: Val::Px(100.),
                    ..default()
                },
            ),
            (
                Name::new("Endgame restart button"),
                Button,
                EndGameRestartButton,
                Node { ..default() },
                BackgroundColor(GNOME_THROW_OUTLINE),
                children![(
                    Text::new(format!("{}\n\n Double Click to Restart", text)),
                    TextFont {
                        font: asset_server.load("fonts/Arbutus-Regular.ttf"),
                        font_size: 36.0,
                        ..default()
                    },
                )]
            )
        ],
    )
}

fn end_game_button_system(
    q_interactions: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            With<EndGameRestartButton>,
        ),
    >,
    mut restart_events: EventWriter<RestartGameEvent>,
) {
    for interaction in q_interactions {
        if *interaction == Interaction::Pressed {
            info!("Press endgame restart button");
            restart_events.write_default();
        }
    }
}
