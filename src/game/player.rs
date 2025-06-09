//! Player-specific behavior.

use crate::PausableSystems;
use crate::asset_tracking::LoadResource;
use crate::game::plant::PlantType;
use crate::game::player_animation::PlayerAnimation;
use crate::theme::palette::PLAYER_THROW_OUTLINE;
use bevy::input::common_conditions::*;
use bevy::window::PrimaryWindow;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_vector_shapes::prelude::*;
use pathfinding::prelude::astar;
use std::f32::consts::TAU;

pub const PLAYER_THROW_RADIUS_PX: f32 = 240.;
pub const PLAYER_THROW_MIN_DIST_PX: f32 = 30.;

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
        Transform::from_translation(Vec3::new(-0.0, 0.0, 1.0)),
        player_animation,
        children![(
            Name::new("Chain"),
            Sprite {
                image: player_assets.chain.clone(),
                ..default()
            },
            Transform {
                translation: Vec3::new(-30., -30., 0.),
                rotation: Quat::from_rotation_z(TAU / 4.),
                scale: Vec3::splat(0.25),
            },
        )],
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
    chain: Handle<Image>,
    #[dependency]
    pub throw_sounds: Vec<Handle<AudioSource>>,
}

#[derive(Event, Debug, Default)]
pub struct PlayerClickEvent(pub Vec2);

#[derive(Event, Debug)]
pub struct ThrowSeedEvent {
    pub from_player: bool,
    pub path: Vec<IVec2>,
    pub seed_type: PlantType,
}

pub fn throw_path(
    origin: IVec2,
    midpoints: Vec<IVec2>,
    destination: IVec2,
    origin_radius: f32,
    midpoint_radius: f32,
) -> Option<Vec<IVec2>> {
    let mut not_origin = midpoints.clone();
    not_origin.push(destination);

    let successors = |p0: &IVec2| -> Vec<(IVec2, i32)> {
        let radius_sq = if *p0 == origin {
            (origin_radius * origin_radius) as i32
        } else {
            (midpoint_radius * midpoint_radius) as i32
        };
        not_origin
            .iter()
            .map(|&p1| (p1, (p1 - p0).length_squared()))
            .filter(|(p1, cost_sq)| p0 != p1 && *cost_sq <= radius_sq)
            .collect()
    };

    let astar_result = astar(
        &origin,
        successors,
        |p| (destination - p).length_squared(),
        |p| *p == destination,
    )?;
    Some(astar_result.0)
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
            chain: assets.load_with_settings(
                "images/chain.png",
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
        painter.color = PLAYER_THROW_OUTLINE;
        painter.transform = *player_pos;
        painter.hollow = true;
        painter.thickness = 1.0;
        painter.circle(PLAYER_THROW_RADIUS_PX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_simple() {
        let origin = IVec2::new(0, 0);
        let midpoints = vec![];
        let dest = IVec2::new(10, 10);

        let path = throw_path(origin, midpoints, dest, 120.0, 0.0);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], IVec2::new(0, 0));
        assert_eq!(path[1], IVec2::new(10, 10));
    }

    #[test]
    fn test_path_complex() {
        let origin = IVec2::new(0, 0);
        let midpoints = vec![IVec2::new(5, 1), IVec2::new(5, 100), IVec2::new(4, 0)];
        let dest = IVec2::new(7, 1);

        let origin_radius = 5.0;
        let midpoint_radius = 3.0;

        let path = throw_path(origin, midpoints, dest, origin_radius, midpoint_radius);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], IVec2::new(0, 0));
        assert_eq!(path[1], IVec2::new(4, 0));
        assert_eq!(path[2], IVec2::new(5, 1));
        assert_eq!(path[3], IVec2::new(7, 1));
    }

    #[test]
    fn test_path_simple_too_far() {
        let origin = IVec2::new(0, 0);
        let midpoints = vec![];
        let dest = IVec2::new(10, 10);

        let path = throw_path(origin, midpoints, dest, 2.0, 0.0);
        assert!(path.is_none());
    }

    #[test]
    fn test_path_complex_too_far() {
        let origin = IVec2::new(0, 0);
        let midpoints = vec![IVec2::new(5, 1), IVec2::new(5, 100), IVec2::new(4, 0)];
        let dest = IVec2::new(7, 1);

        let origin_radius = 4.0;
        let midpoint_radius = 1.0;

        let path = throw_path(origin, midpoints, dest, origin_radius, midpoint_radius);
        print!("{:?}", path);
        assert!(path.is_none());
    }
}
