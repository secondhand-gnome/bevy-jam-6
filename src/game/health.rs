use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Health>();

    app.register_type::<HealthAssets>();
    app.load_resource::<HealthAssets>();

    app.add_systems(Update, remove_dead.run_if(resource_exists::<HealthAssets>));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Health {
    current: i32,
    max: i32,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct HealthAssets {
    #[dependency]
    death_sound: Handle<AudioSource>,
}

impl FromWorld for HealthAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            death_sound: assets.load("audio/sound_effects/death.ogg"),
        }
    }
}

impl Health {
    pub fn new(amount: i32) -> Self {
        Self {
            current: amount,
            max: amount,
        }
    }

    pub fn reduce(&mut self, amount: i32) {
        self.current = std::cmp::max(0, self.current - amount);
    }

    fn is_alive(&self) -> bool {
        self.current > 0
    }
}

fn remove_dead(
    mut commands: Commands,
    q_health: Query<(Entity, &Health, &Transform)>,
    health_assets: Res<HealthAssets>,
) {
    for (entity, health, transform) in q_health {
        if !health.is_alive() {
            info!("{:?} dies", entity);
            commands.entity(entity).despawn();

            // Play sound
            commands.spawn((
                sound_effect(health_assets.death_sound.clone()),
                Transform::from_translation(transform.translation),
            ));

            // TODO spawn particle effects
        }
    }
}
