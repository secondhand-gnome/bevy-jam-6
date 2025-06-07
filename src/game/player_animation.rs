//! Player sprite animation.

use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::game::player::{Player, ThrowSeedEvent};
use crate::{AppSystems, PausableSystems, audio::sound_effect, game::player::PlayerAssets};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.register_type::<PlayerAnimation>();
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (animate_throw_seed, update_animation_atlas)
                .chain()
                .run_if(resource_exists::<PlayerAssets>)
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

fn animate_throw_seed(
    mut commands: Commands,
    mut events: EventReader<ThrowSeedEvent>,
    mut player_animation: Single<&mut PlayerAnimation, With<Player>>,
    player_assets: Res<PlayerAssets>,
) {
    for ev in events.read() {
        if !ev.from_player {
            continue;
        }
        let rng = &mut thread_rng();
        let throw_sound = player_assets.throw_sounds.choose(rng).unwrap().clone();
        let Some(throw_origin) = ev.path.first() else {
            warn!("No origin for throw path");
            continue;
        };
        commands.spawn((
            sound_effect(throw_sound),
            Transform::from_translation(throw_origin.as_vec2().extend(0.)),
        ));

        let mut left = false;
        if let Some(next_point) = ev.path.get(1) {
            left = throw_origin.x > next_point.x;
        }
        player_animation.update_state(PlayerAnimationState::Planting(left));
    }
}

fn update_animation_timer(time: Res<Time>, mut query: Query<&mut PlayerAnimation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

fn update_animation_atlas(mut query: Query<(&PlayerAnimation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        atlas.index = animation.frame();
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: PlayerAnimationState,
}

#[derive(Reflect, PartialEq)]
pub enum PlayerAnimationState {
    Idling,
    Planting(bool),
    Mailing,
}

impl PlayerAnimation {
    const IDLE_FRAME: usize = 0;
    const IDLE_INTERVAL: Duration = Duration::from_millis(500);
    const PLANTING_FRAME_RIGHT: usize = 2;
    const PLANTING_FRAME_LEFT: usize = 1;
    const PLANTING_INTERVAL: Duration = Duration::from_millis(500);
    const MAILING_FRAME: usize = 1;
    const MAILING_INTERVAL: Duration = Duration::from_millis(250);

    pub fn frame(&self) -> usize {
        self.frame
    }

    fn idling() -> Self {
        Self {
            timer: Timer::new(Self::IDLE_INTERVAL, TimerMode::Once),
            frame: Self::IDLE_FRAME,
            state: PlayerAnimationState::Idling,
        }
    }

    fn planting(left: bool) -> Self {
        let frame = if left {
            Self::PLANTING_FRAME_LEFT
        } else {
            Self::PLANTING_FRAME_RIGHT
        };
        Self {
            timer: Timer::new(Self::PLANTING_INTERVAL, TimerMode::Once),
            frame,
            state: PlayerAnimationState::Planting(left),
        }
    }

    fn mailing() -> Self {
        Self {
            timer: Timer::new(Self::MAILING_INTERVAL, TimerMode::Once),
            frame: Self::MAILING_FRAME,
            state: PlayerAnimationState::Mailing,
        }
    }

    pub fn new() -> Self {
        Self::idling()
    }

    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.finished() {
            self.update_state(PlayerAnimationState::Idling);
        }
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: PlayerAnimationState) {
        if self.state != state {
            match state {
                PlayerAnimationState::Idling => *self = Self::idling(),
                PlayerAnimationState::Planting(left) => *self = Self::planting(left),
                PlayerAnimationState::Mailing => *self = Self::mailing(),
            }
        }
    }
}
