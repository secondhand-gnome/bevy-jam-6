//! Enemies eat plants.

use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::game::despawn::DespawnOnRestart;
use crate::game::health::Health;
use crate::game::lifespan::LifespanTimer;
use crate::game::physics::GameLayer;
use crate::game::plant::{
    Burnable, DRAGONFRUIT_STRENGTH, DamagePlantEvent, GNOME_STRENGTH, GrowthTimer,
    PINEAPPLE_MAX_GENERATION, PINEAPPLE_SPREAD_DISTANCE, PINEAPPLE_STRENGTH, Plant, PlantType,
    SowPlantEvent, SpewFireEvent,
};
use crate::game::player::Player;
use crate::theme::palette::ENEMY_EAT_OUTLINE;
use crate::{OnPauseSystems, PausableSystems};
use avian2d::math::TAU;
use avian2d::prelude::*;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::math::ops::{cos, sin};
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use rand::Rng;
use rand::prelude::SliceRandom;

const ENEMY_RADIUS: f32 = 30.0;
const ENEMY_DESPAWN_DISTANCE: f32 = 1500.0;
const EAT_RADIUS_PX: f32 = 80.0;
const SPAWN_INTERVAL_S: f32 = 1.0;

const ENEMY_MOVE_SPEED: f32 = 120.0;
const ENEMY_SPAWN_LIMIT: usize = 3;

const BITE_COOLDOWN_S: f32 = 2.5;
const BITE_STRENGTH: i32 = 1;
const ENEMY_MAX_HEALTH: i32 = 5;

const STAR_LIFETIME_S: f32 = 0.25;
const STAR_SCALE: f32 = 0.25;
const STAR_Z_LAYER: f32 = 2.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();

    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();

    app.add_event::<DamageEnemyEvent>();

    app.add_systems(Update, freeze_enemies.in_set(OnPauseSystems));
    app.add_systems(
        Update,
        (
            tick_spawn,
            tick_bite_cooldowns,
            pursue_plants,
            damage_enemies,
        )
            .run_if(resource_exists::<EnemyAssets>)
            .in_set(PausableSystems),
    );

    app.add_systems(Update, draw_eat_radius);
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
        DespawnOnRestart,
        Collider::circle(ENEMY_RADIUS),
        CollisionLayers::new(
            [GameLayer::Enemy],
            [GameLayer::Plant, GameLayer::Enemy, GameLayer::Fireball],
        ),
        Sprite {
            image: enemy_assets.rat.clone(),
            ..default()
        },
        Burnable,
        Health::new(ENEMY_MAX_HEALTH),
        Transform::from_translation(spawn_position),
        children![(
            Name::new("Enemy eat collider"),
            Collider::circle(EAT_RADIUS_PX),
            CollisionLayers::new(LayerMask::NONE, [GameLayer::Plant]),
        )],
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
    #[dependency]
    rat_dead: Handle<Image>,
    #[dependency]
    rat_hit: Handle<Image>,
    #[dependency]
    rat_walk: Handle<Image>,
    #[dependency]
    star_particles: Vec<Handle<Image>>,
    #[dependency]
    bite_sounds: Vec<Handle<AudioSource>>,
    #[dependency]
    rat_damage_sound: Handle<AudioSource>,
    #[dependency]
    headbonk_sound: Handle<AudioSource>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component)]
struct EnemySpawner {
    spawn_height: f32,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct SpawnTimer(Timer);

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct BiteCooldown(Timer);

#[derive(Event, Debug)]
pub struct DamageEnemyEvent {
    pub enemy_entity: Entity,
    pub amount: i32,
    pub position: Vec3,
}

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
            star_particles: vec![
                "images/particles/star/star_04.png",
                "images/particles/star/star_05.png",
            ]
            .into_iter()
            .map(|path| {
                assets.load_with_settings(path, |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                })
            })
            .collect::<Vec<Handle<Image>>>(),
            bite_sounds: vec![
                assets.load("audio/sound_effects/bite/bite1.ogg"),
                assets.load("audio/sound_effects/bite/bite2.ogg"),
                assets.load("audio/sound_effects/bite/bite3.ogg"),
            ],
            rat_damage_sound: assets.load("audio/sound_effects/rat_damage.ogg"),
            headbonk_sound: assets.load("audio/sound_effects/headbonk.ogg"),
        }
    }
}

fn freeze_enemies(mut q_enemies: Query<&mut LinearVelocity, With<Enemy>>) {
    for mut vel in q_enemies.iter_mut() {
        *vel = LinearVelocity::ZERO;
    }
}

fn tick_spawn(
    mut commands: Commands,
    q_enemy_spawners: Query<(&Transform, &mut SpawnTimer, &EnemySpawner)>,
    q_enemies: Query<&Enemy>,
    q_plants: Query<&Plant>,
    time: Res<Time>,
    enemy_assets: Res<EnemyAssets>,
) {
    for (transform, mut spawn_timer, enemy_spawner) in q_enemy_spawners {
        spawn_timer.0.tick(time.delta());

        if spawn_timer.0.just_finished() {
            if q_enemies.iter().len() >= ENEMY_SPAWN_LIMIT {
                debug!("Not spawning an enemy - limit reached");
                return;
            }
            if q_plants
                .iter()
                .filter(|p| p.plant_type() == PlantType::Daisy)
                .count()
                == 0
            {
                debug!("Not spawning an enemy - no daisies");
                return;
            }

            let rng = &mut rand::thread_rng();
            let rand_f32: f32 = rng.r#gen();
            let y_offset = (rand_f32 - 0.5) * enemy_spawner.spawn_height;
            let mut spawn_position = transform.translation;
            spawn_position.y += y_offset;

            info!(
                "Spawning an enemy at {:?}",
                transform.translation + spawn_position
            );
            commands.spawn(enemy(spawn_position, &enemy_assets));
        }
    }
}

fn pursue_plants(
    mut commands: Commands,
    mut q_enemies: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            Option<&BiteCooldown>,
        ),
        With<Enemy>,
    >,
    q_plants: Query<(Entity, &Transform, &Plant, Option<&GrowthTimer>)>,
    q_player: Query<&Transform, With<Player>>,
    mut damage_plant_events: EventWriter<DamagePlantEvent>,
    mut damage_enemy_events: EventWriter<DamageEnemyEvent>,
    mut sow_plant_events: EventWriter<SowPlantEvent>,
    mut spew_fire_events: EventWriter<SpewFireEvent>,
    enemy_assets: Res<EnemyAssets>,
) {
    let Ok(player_transform) = q_player.single() else {
        return;
    };
    for (enemy, enemy_transform, mut enemy_velocity, optional_bite_cooldown) in q_enemies.iter_mut()
    {
        let dist_from_player =
            (enemy_transform.translation - player_transform.translation).length();
        if dist_from_player > ENEMY_DESPAWN_DISTANCE {
            info!("Despawning enemy {:?} due to distance", enemy);
            commands.entity(enemy).try_despawn();
            return;
        }

        let mut plant_vectors: Vec<_> = q_plants
            .iter()
            .map(|(entity, plant_transform, plant, opt_growth_timer)| {
                (
                    entity,
                    plant_transform.translation - enemy_transform.translation,
                    plant,
                    plant_transform,
                    opt_growth_timer.is_some(),
                )
            })
            .collect();

        if plant_vectors.is_empty() {
            // No plants - Move up
            *enemy_velocity = LinearVelocity(ENEMY_MOVE_SPEED * Vec2::new(0., 1.));
            continue;
        }

        // Sort plants by distance from this enemy
        plant_vectors.sort_by(|a, b| a.1.length().partial_cmp(&b.1.length()).unwrap());
        let (plant_entity, plant_vector, plant, plant_transform, is_growing) =
            plant_vectors.first().unwrap();

        if plant_vector.length() < EAT_RADIUS_PX {
            // Eat the plant
            if optional_bite_cooldown.is_some() {
                // In cooldown - cannot bite
            } else {
                // Bite
                info!("Bite plant {:?}", plant_entity);

                damage_plant_events.write(DamagePlantEvent {
                    plant_entity: *plant_entity,
                    amount: BITE_STRENGTH,
                });

                if !is_growing {
                    match plant.plant_type() {
                        PlantType::Daisy => {
                            // Do nothing
                        }
                        PlantType::Pineapple(generation) => {
                            // Enemy takes damage
                            damage_enemy_events.write(DamageEnemyEvent {
                                enemy_entity: enemy,
                                amount: PINEAPPLE_STRENGTH,
                                position: enemy_transform.translation,
                            });

                            commands.spawn((
                                sound_effect(enemy_assets.rat_damage_sound.clone()),
                                Transform::from_translation(enemy_transform.translation),
                            ));

                            let rng = &mut rand::thread_rng();
                            let angle: f32 = rng.gen_range(0.0..TAU);
                            let spawn_vec2 = PINEAPPLE_SPREAD_DISTANCE
                                * Vec2::new(cos(angle), sin(angle)).normalize();
                            let spawn_pos = plant_transform.translation.xy() + spawn_vec2;

                            if generation < PINEAPPLE_MAX_GENERATION {
                                info!("Spawning new pineapple (generation={})", generation + 1);
                                sow_plant_events.write(SowPlantEvent {
                                    position: spawn_pos,
                                    seed_type: PlantType::Pineapple(generation + 1),
                                });
                            }
                        }
                        PlantType::Dragonfruit => {
                            // Enemy takes damage
                            damage_enemy_events.write(DamageEnemyEvent {
                                enemy_entity: enemy,
                                amount: DRAGONFRUIT_STRENGTH,
                                position: enemy_transform.translation,
                            });

                            spew_fire_events.write(SpewFireEvent {
                                plant_entity: *plant_entity,
                                origin: plant_transform.translation,
                            });

                            commands.spawn((
                                sound_effect(enemy_assets.rat_damage_sound.clone()),
                                Transform::from_translation(enemy_transform.translation),
                            ));
                        }
                        PlantType::Gnome => {
                            // Enemy takes damage
                            damage_enemy_events.write(DamageEnemyEvent {
                                enemy_entity: enemy,
                                amount: GNOME_STRENGTH,
                                position: enemy_transform.translation,
                            });
                            commands.spawn((
                                sound_effect(enemy_assets.headbonk_sound.clone()),
                                Transform::from_translation(enemy_transform.translation),
                            ));
                        }
                    }
                }

                commands
                    .entity(enemy)
                    .insert(BiteCooldown(Timer::from_seconds(
                        BITE_COOLDOWN_S,
                        TimerMode::Once,
                    )));

                // Play bite sound
                let rng = &mut rand::thread_rng();
                let random_bite_sound = enemy_assets.bite_sounds.choose(rng).unwrap().clone();
                commands.spawn((
                    sound_effect(random_bite_sound),
                    Transform::from_translation(enemy_transform.translation),
                ));
            }
        } else {
            // Move towards the plant
            *enemy_velocity = LinearVelocity(ENEMY_MOVE_SPEED * plant_vector.xy().normalize());
        }
    }
}

fn draw_eat_radius(mut painter: ShapePainter, q_enemies: Query<&Transform, With<Enemy>>) {
    painter.hollow = true;
    painter.thickness = 0.25;
    painter.color = ENEMY_EAT_OUTLINE;
    for enemy_transform in q_enemies {
        painter.transform.translation = enemy_transform.translation;
        painter.circle(EAT_RADIUS_PX);
    }
}

fn tick_bite_cooldowns(
    mut commands: Commands,
    mut q_bite_cooldowns: Query<(Entity, &mut BiteCooldown)>,
    time: Res<Time>,
) {
    for (entity, mut cooldown) in &mut q_bite_cooldowns {
        cooldown.0.tick(time.delta());

        if cooldown.0.finished() {
            commands.entity(entity).remove::<BiteCooldown>();
        }
    }
}

fn damage_enemies(
    mut commands: Commands,
    mut q_enemies: Query<(Entity, &mut Health), With<Enemy>>,
    mut damage_enemy_events: EventReader<DamageEnemyEvent>,
    enemy_assets: Res<EnemyAssets>,
) {
    for ev in damage_enemy_events.read() {
        for (entity, mut health) in q_enemies.iter_mut() {
            if entity == ev.enemy_entity {
                commands.spawn((
                    Name::new("Star particle"),
                    DespawnOnRestart,
                    LifespanTimer(Timer::from_seconds(STAR_LIFETIME_S, TimerMode::Once)),
                    Sprite {
                        image: random_star_particle(&enemy_assets),
                        ..default()
                    },
                    Transform::from_translation(ev.position.with_z(STAR_Z_LAYER))
                        .with_scale(Vec3::splat(STAR_SCALE)),
                ));
                health.reduce(ev.amount);
                info!(
                    "Damage enemy {:?} for {} (now at {:?})",
                    entity, ev.amount, health
                );
            }
        }
    }
}

fn random_star_particle(enemy_assets: &EnemyAssets) -> Handle<Image> {
    let rng = &mut rand::thread_rng();
    enemy_assets.star_particles.choose(rng).unwrap().clone()
}
