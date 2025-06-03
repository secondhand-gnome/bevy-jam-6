use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::audio::sound_effect;
use crate::theme::palette::{HEALTH_HIGH, HEALTH_LOW, HEALTH_MED, HEALTH_OUTLINE};
use bevy::prelude::*;
use bevy_vector_shapes::painter::ShapePainter;
use bevy_vector_shapes::prelude::RectPainter;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Health>();

    app.register_type::<HealthAssets>();
    app.load_resource::<HealthAssets>();

    app.add_systems(Update, draw_health);

    app.add_systems(
        Update,
        remove_dead
            .run_if(resource_exists::<HealthAssets>)
            .in_set(PausableSystems),
    );
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

    /// Return the fraction of remaining health from 0.0 to 1.0
    pub fn fraction(&self) -> f32 {
        (self.current as f32) / (self.max as f32)
    }

    pub fn bar_color(&self) -> Color {
        let frac = self.fraction();
        if frac < 0.25 {
            HEALTH_LOW
        } else if frac < 0.6 {
            HEALTH_MED
        } else {
            HEALTH_HIGH
        }
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

fn draw_health(mut painter: ShapePainter, q_creatures: Query<(&Transform, &Health)>) {
    const HEALTH_HEIGHT_PX: f32 = 30. * 0.2;
    const HEALTH_LENGTH_PX: f32 = 30. * 1.;
    const HEALTH_DIMENS: Vec2 = Vec2::new(HEALTH_LENGTH_PX, HEALTH_HEIGHT_PX);
    const HEALTH_OFFSET: Vec3 = Vec3::new(0., 1.1 * 30., 0.);

    for (transform, health) in q_creatures {
        // Draw the remaining health
        painter.transform.translation = transform.translation + HEALTH_OFFSET;
        painter.hollow = true;
        painter.thickness = 0.5;
        painter.color = HEALTH_OUTLINE;
        painter.rect(HEALTH_DIMENS);

        painter.hollow = false;
        painter.color = health.bar_color();
        painter.rect(Vec2::new(
            HEALTH_DIMENS.x * health.fraction(),
            HEALTH_DIMENS.y * 0.8,
        ));
    }
}
