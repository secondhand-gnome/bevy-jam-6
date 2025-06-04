use crate::game::farm::{BankAccount, BankAccountUpdateEvent};
use crate::game::plant::{PlantType, SeedSelection};
use bevy::prelude::*;
use bevy::time::common_conditions;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(CobwebUiPlugin).load("ui/hello.cobweb");

    app.add_systems(Update, update_ui);
}

struct BalanceUpdate;

pub fn build_ui(mut commands: Commands, mut scene_builder: SceneBuilder) {
    commands
        .ui_root()
        .spawn_scene(("ui/hello.cobweb", "scene"), &mut scene_builder, |h| {
            h.insert_reactive(SeedSelection::default());
            let scene_entity = h.id();

            h.edit("seed_button_daisy", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Daisy);
                        OK
                    },
                );
            });
            h.edit("seed_button_pineapple", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Pineapple);
                        OK
                    },
                );
            });
            h.edit("seed_button_dragonfruit", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Dragonfruit);
                        OK
                    },
                );
            });
            h.edit("seed_button_gnome", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Gnome);
                        OK
                    },
                );
            });

            h.get("bank").update_on(
                broadcast::<BalanceUpdate>(),
                move |id: TargetId, mut editor: TextEditor, q_bank_account: Query<&BankAccount>| {
                    let Ok(bank_account) = q_bank_account.single() else {
                        warn!("No bank account");
                        return;
                    };
                    let balance = bank_account.balance();
                    write_text!(editor, *id, "Bank balance: ${}", balance);
                    info!("Update UI for bank account balance {:?}", balance);
                },
            );
        });
}

// TODO hide UI on exit

fn update_ui(
    mut commands: Commands,
    mut bank_account_update_events: EventReader<BankAccountUpdateEvent>,
) {
    for _ in bank_account_update_events.read() {
        commands.react().broadcast(BalanceUpdate);
    }
}
