//! Development tools for the game. This plugin is only enabled in dev builds.

use crate::screens::Screen;
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
    ui::UiDebugOptions,
};
// use bevy_inspector_egui::bevy_egui::EguiPlugin;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub(super) fn plugin(app: &mut App) {
    // bevy-inspector-egui
    // app.add_plugins(EguiPlugin {
    //     enable_multipass_for_primary_context: true,
    // })
    // .add_plugins(WorldInspectorPlugin::new());

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
