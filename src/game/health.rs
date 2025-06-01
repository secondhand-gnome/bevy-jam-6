use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Health>();

    app.add_systems(Update, remove_dead);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Health {
    current: i32,
    max: i32,
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

fn remove_dead(mut commands: Commands, q_health: Query<(Entity, &Health)>) {
    for (entity, health) in q_health {
        if !health.is_alive() {
            info!("{:?} dies", entity);
            // TODO spawn death animation and sound
            commands.entity(entity).despawn();
        }
    }
}
