use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::game::farm::{BankAccount, BankAccountUpdateEvent};
use crate::game::health::Health;
use crate::game::physics::GameLayer;
use crate::theme::palette::{PLANT_GROWTH_FOREGROUND, PLANT_GROWTH_OUTLINE};
use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, CollisionStarted, LinearVelocity, RigidBody,
};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::prelude::*;
use rand::prelude::SliceRandom;

const PLANT_RADIUS_PX: f32 = 30.;
const DAISY_GROWTH_TIME_S: f32 = 3.;
const PLANT_MAX_HEALTH: i32 = 5; // TODO depends on plant type

const DAISY_CHAIN_LENGTH: usize = 3;
// const DAISY_CHAIN_DISTANCE: f32 = 40.; // TODO use
const DAISY_CHAIN_VALUE: f32 = 10.;

pub const GNOME_STRENGTH: i32 = 1;
pub const PINEAPPLE_STRENGTH: i32 = 2;
pub const DRAGONFRUIT_STRENGTH: i32 = 1;

const FIREBALL_RADIUS_PX: f32 = 30.;
const FIREBALL_START_OFFSET_PX: f32 = 40.;
const FIREBALL_LIFETIME_S: f32 = 1.0;
const FIREBALL_MOVE_SPEED: f32 = 15.0;
const FIREBALL_DAMAGE: i32 = 2;

// Prices are set both here and in the .cobweb file
const DAISY_PRICE: f32 = 1.;
const PINEAPPLE_PRICE: f32 = 2.;
const DRAGONFRUIT_PRICE: f32 = 3.;
const GNOME_PRICE: f32 = 5.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Plant>();

    app.register_type::<PlantAssets>();
    app.load_resource::<PlantAssets>();

    app.add_event::<SowPlantEvent>();
    app.add_event::<DamagePlantEvent>();
    app.add_event::<SpewFireEvent>();
    app.add_event::<SellDaisyChainEvent>();

    app.add_systems(
        Update,
        (
            sow_plants,
            damage_plants,
            tick_growth,
            spew_fire,
            tick_fireball_lifetime,
            burn_stuff,
            form_daisy_chains,
            sell_daisy_chains,
        )
            .run_if(resource_exists::<PlantAssets>)
            .in_set(PausableSystems),
    );
    app.add_systems(Update, (draw_plant_circles, draw_growth));
}

// TODO a Gnome plant should spawn a Gnome entity at the same transform, which will handle throwing/catching
fn plant(position: Vec2, plant_assets: &PlantAssets, plant_type: PlantType) -> impl Bundle {
    (
        Name::new(format!("Plant at {:?}", position)),
        Plant { plant_type },
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

fn fireball(
    spawning_entity: Entity,
    origin: Vec3,
    direction: Vec2,
    plant_assets: &PlantAssets,
) -> impl Bundle {
    (
        Name::new("Fireball"),
        Fireball {
            active: true,
            spawning_entity,
        },
        RigidBody::Kinematic,
        Collider::circle(FIREBALL_RADIUS_PX),
        CollisionLayers::new([GameLayer::Fireball], [GameLayer::Enemy]), // TODO also interact with plants
        CollisionEventsEnabled,
        Sprite {
            image: plant_assets.fireball.clone(),
            ..default()
        },
        FireballLifeTimer(Timer::from_seconds(FIREBALL_LIFETIME_S, TimerMode::Once)),
        Transform::from_translation(
            origin + FIREBALL_START_OFFSET_PX * direction.extend(0.).normalize(),
        ),
        LinearVelocity(direction),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Plant {
    plant_type: PlantType,
}

impl Plant {
    pub fn plant_type(&self) -> PlantType {
        self.plant_type
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct GrowthTimer(Timer);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Fireball {
    active: bool,
    spawning_entity: Entity,
}

impl Fireball {
    fn deactivate(&mut self) {
        self.active = false;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Burnable;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct FireballLifeTimer(Timer);

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
    gnome: Handle<Image>,
    #[dependency]
    fireball: Handle<Image>,
    #[dependency]
    sow_sounds: Vec<Handle<AudioSource>>,
    #[dependency]
    growth_sound: Handle<AudioSource>,
    #[dependency]
    death_sound: Handle<AudioSource>,
    #[dependency]
    fireball_spawn_sound: Handle<AudioSource>,
    #[dependency]
    burn_sound: Handle<AudioSource>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum PlantType {
    #[default]
    Daisy,
    Pineapple,
    Dragonfruit,
    Gnome,
}

impl PlantType {
    pub fn price(&self) -> f32 {
        match self {
            PlantType::Daisy => DAISY_PRICE,
            PlantType::Pineapple => PINEAPPLE_PRICE,
            PlantType::Dragonfruit => DRAGONFRUIT_PRICE,
            PlantType::Gnome => GNOME_PRICE,
        }
    }
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

#[derive(Event, Debug)]
pub struct DamagePlantEvent {
    pub plant_entity: Entity,
    pub amount: i32,
}

#[derive(Event, Debug)]
pub struct SpewFireEvent {
    pub plant_entity: Entity,
    pub origin: Vec3,
}

#[derive(Event, Debug)]
pub struct SellDaisyChainEvent {
    daisy_entities: Vec<Entity>,
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
            gnome: assets.load_with_settings(
                "images/plants/gnome.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            fireball: assets.load_with_settings(
                "images/fireball.png",
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
            fireball_spawn_sound: assets.load("audio/sound_effects/fireball_spawn.ogg"),
            burn_sound: assets.load("audio/sound_effects/burn.ogg"),
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
        commands.spawn(plant(event.position, &plant_assets, event.seed_type));

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

fn tick_growth(
    mut commands: Commands,
    mut q_growing_plants: Query<(Entity, &Plant, &mut Transform, &mut GrowthTimer)>,
    time: Res<Time>,
    plant_assets: Res<PlantAssets>,
) {
    for (entity, plant, mut transform, mut growth_timer) in &mut q_growing_plants {
        growth_timer.0.tick(time.delta());
        if growth_timer.0.finished() {
            commands
                .entity(entity)
                .remove::<GrowthTimer>()
                .remove::<Sprite>()
                .insert(Sprite {
                    image: match plant.plant_type {
                        PlantType::Daisy => plant_assets.daisy.clone(),
                        PlantType::Pineapple => plant_assets.pineapple.clone(),
                        PlantType::Dragonfruit => plant_assets.dragonfruit.clone(),
                        PlantType::Gnome => plant_assets.gnome.clone(),
                    },
                    custom_size: match plant.plant_type {
                        PlantType::Daisy => None,
                        PlantType::Pineapple => Some(Vec2::splat(64.)),
                        PlantType::Dragonfruit => Some(Vec2::splat(64.)),
                        PlantType::Gnome => Some(Vec2::splat(64.)),
                    },
                    ..default()
                });

            transform.scale = Vec3::splat(0.5);

            commands.spawn((
                sound_effect(plant_assets.growth_sound.clone()),
                Transform::from_translation(transform.translation),
            ));

            println!("Plant {:?} finished growing", entity);
        }
    }
}

fn tick_fireball_lifetime(
    mut commands: Commands,
    q_fireballs: Query<(Entity, &mut FireballLifeTimer), With<Fireball>>,
    time: Res<Time>,
) {
    for (entity, mut timer) in q_fireballs {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn burn_stuff(
    mut commands: Commands,
    mut q_fireballs: Query<(Entity, &Transform, &mut Fireball)>,
    mut q_burnables: Query<(Entity, &mut Health, &Transform), With<Burnable>>,
    mut collision_event_reader: EventReader<CollisionStarted>,
    plant_assets: Res<PlantAssets>,
) {
    if collision_event_reader.is_empty() {
        return;
    }
    let fireball_entities: Vec<Entity> = q_fireballs
        .iter()
        .filter(|(_, _, f)| f.active)
        .map(|(e, _, _)| e)
        .collect();
    let burnable_entities: Vec<Entity> = q_burnables.iter().map(|(e, _, _)| e).collect();
    for CollisionStarted(entity1, entity2) in collision_event_reader.read() {
        let (fireball_entity, burnable_entity) =
            if fireball_entities.contains(entity1) && burnable_entities.contains(entity2) {
                (entity1, entity2)
            } else if burnable_entities.contains(entity1) && fireball_entities.contains(entity2) {
                (entity2, entity1)
            } else {
                continue;
            };
        let Some(mut burnable_health) = q_burnables
            .iter_mut()
            .find(|(e, _, _)| e == burnable_entity)
            .map(|(_, h, _)| h)
        else {
            continue;
        };

        let Some(mut fireball) = q_fireballs
            .iter_mut()
            .find(|(e, _, _)| e == fireball_entity)
            .map(|(_, _, f)| f)
        else {
            continue;
        };

        // TODO write a DamageEnemyEvent instead of directly affecting health here
        burnable_health.reduce(FIREBALL_DAMAGE);
        commands.entity(*fireball_entity).despawn();
        fireball.deactivate();

        // TODO make smoke
        commands.spawn((
            sound_effect(plant_assets.burn_sound.clone()),
            // TODO transform
            // Transform::from_translation(event.position.extend(0.)),
        ));
    }
}

fn form_daisy_chains(
    q_plants: Query<(Entity, &Transform, &Plant), Without<GrowthTimer>>,
    mut sell_events: EventWriter<SellDaisyChainEvent>,
) {
    let daisies: Vec<_> = q_plants
        .iter()
        .filter(|(_, _, p)| p.plant_type == PlantType::Daisy)
        .collect();

    // Find 3 daisies next to each other
    // TODO implement "closest triplet" algorithm
    // TODO For now just pick any 3
    if daisies.len() < DAISY_CHAIN_LENGTH {
        return;
    }
    let daisy_entities: Vec<_> = daisies
        .iter()
        .map(|(e, _, _)| *e)
        .take(DAISY_CHAIN_LENGTH)
        .collect();
    sell_events.write(SellDaisyChainEvent { daisy_entities });
}

fn sell_daisy_chains(
    mut commands: Commands,
    mut sell_events: EventReader<SellDaisyChainEvent>,
    mut q_bank_account: Query<&mut BankAccount>,
    mut bank_account_update_events: EventWriter<BankAccountUpdateEvent>,
) {
    for ev in sell_events.read() {
        info!("Selling daisy chain: {:?}", ev.daisy_entities.iter());
        for entity in ev.daisy_entities.iter() {
            // TODO instead of despawn, mark as "Sold" and give it a short lifetime
            // TODO enemies should ignore "Sold" daisies
            commands.entity(*entity).despawn();
        }
        let Ok(mut bank_account) = q_bank_account.single_mut() else {
            warn!("No bank account!");
            return;
        };
        bank_account.credit(DAISY_CHAIN_VALUE);
        bank_account_update_events.write(BankAccountUpdateEvent);
        // TODO sound, animation, particle effects
    }
}

fn damage_plants(
    mut q_plants: Query<(Entity, &mut Health), With<Plant>>,
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

fn spew_fire(
    mut commands: Commands,
    mut spew_fire_events: EventReader<SpewFireEvent>,
    plant_assets: Res<PlantAssets>,
) {
    for ev in spew_fire_events.read() {
        const DIRECTIONS: [Vec2; 4] = [
            Vec2::new(FIREBALL_MOVE_SPEED, 0.),
            Vec2::new(-FIREBALL_MOVE_SPEED, 0.),
            Vec2::new(0., FIREBALL_MOVE_SPEED),
            Vec2::new(0., -FIREBALL_MOVE_SPEED),
        ];
        for direction in DIRECTIONS {
            commands.spawn(fireball(
                ev.plant_entity,
                ev.origin,
                direction,
                &plant_assets,
            ));
        }
        commands.spawn((
            sound_effect(plant_assets.fireball_spawn_sound.clone()),
            Transform::from_translation(ev.origin),
        ));
    }
}
