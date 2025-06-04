//! Player-specific behavior.

use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::game::player_animation::PlayerAnimation;
use bevy::input::common_conditions::*;
use bevy::window::PrimaryWindow;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_vector_shapes::prelude::*;

const PLAYER_THROW_RADIUS_PX: f32 = 240.;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    app.add_systems(
        Update,
        on_click
            .run_if(input_just_pressed(MouseButton::Left))
            .in_set(PausableSystems),
    );
    app.add_systems(Update, check_touch.in_set(PausableSystems));

    app.add_systems(
        Update,
        draw_player_circle.run_if(resource_exists::<PlayerAssets>),
    );
}

/// The player character.
pub fn player(
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 3, 1, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();
    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.farmer.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.frame(),
            }),
            ..default()
        },
        Transform::from_translation(Vec3::new(-350.0, 0.0, 1.0)),
        player_animation,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    farmer: Handle<Image>,
    #[dependency]
    pub throw_sounds: Vec<Handle<AudioSource>>,
}

#[derive(Event, Debug, Default)]
pub struct PlayerClickEvent(pub Vec2);

#[derive(Event, Debug, Default)]
// TODO track who is the origin of the throw
pub struct ThrowSeedEvent {
    pub origin: Vec2,
}

pub fn can_player_reach(player_position: Vec2, hit_position: Vec2) -> bool {
    let difference = player_position - hit_position;
    difference.length() < PLAYER_THROW_RADIUS_PX
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            farmer: assets.load_with_settings(
                "images/farmer.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            throw_sounds: vec![
                assets.load("audio/sound_effects/woosh/woosh1.ogg"),
                assets.load("audio/sound_effects/woosh/woosh2.ogg"),
                assets.load("audio/sound_effects/woosh/woosh3.ogg"),
                assets.load("audio/sound_effects/woosh/woosh4.ogg"),
                assets.load("audio/sound_effects/woosh/woosh5.ogg"),
                assets.load("audio/sound_effects/woosh/woosh6.ogg"),
                assets.load("audio/sound_effects/woosh/woosh7.ogg"),
                assets.load("audio/sound_effects/woosh/woosh8.ogg"),
            ],
        }
    }
}

fn on_click(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_transform: Query<&Transform, With<Camera>>,
    mut events: EventWriter<PlayerClickEvent>,
) {
    if let Ok(window) = q_windows.single() {
        if let Ok(transform) = q_transform.single() {
            if let Some(window_position) = window.cursor_position() {
                let world_position = window_to_world(window_position, window, transform);
                events.write(PlayerClickEvent(world_position));
            }
        }
    }
}

fn check_touch(
    touches: Res<Touches>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_transform: Query<&Transform, With<Camera>>,
    mut events: EventWriter<PlayerClickEvent>,
) {
    if let Some(window_position) = touches.first_pressed_position() {
        if let Ok(window) = q_windows.single() {
            if let Ok(transform) = q_transform.single() {
                let world_position = window_to_world(window_position, window, transform);
                events.write(PlayerClickEvent(world_position));
            }
        }
    }
}

fn window_to_world(position: Vec2, window: &Window, camera: &Transform) -> Vec2 {
    let norm = Vec3::new(
        position.x - window.width() / 2.,
        -1. * (position.y - window.height() / 2.),
        0.,
    );

    let world_pos_3d = *camera * norm;
    Vec2::new(world_pos_3d.x, world_pos_3d.y)
}

fn draw_player_circle(mut painter: ShapePainter, q_player: Query<&Transform, With<Player>>) {
    if let Ok(player_pos) = q_player.single() {
        painter.transform = *player_pos;
        painter.hollow = true;
        painter.thickness = 1.0;
        painter.circle(PLAYER_THROW_RADIUS_PX);
    }
}
