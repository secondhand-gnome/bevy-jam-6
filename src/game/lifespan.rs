use crate::PausableSystems;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LifespanTimer>();

    app.add_systems(Update, tick_lifespans.in_set(PausableSystems));
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct LifespanTimer(pub Timer);

fn tick_lifespans(
    mut commands: Commands,
    q_lifespans: Query<(Entity, &mut LifespanTimer)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in q_lifespans {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).try_despawn();
        }
    }
}
