use crate::game::farm::{BankAccount, BankAccountUpdateEvent, RestartGameEvent, WINNING_BALANCE};
use crate::game::plant::{PlantType, SeedSelection};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(CobwebUiPlugin).load("ui/hello.cobweb");

    app.add_systems(Update, update_ui);
}

struct BalanceUpdate;
struct CurrentSeedUpdate;

pub fn build_ui(mut commands: Commands, mut scene_builder: SceneBuilder) {
    commands
        .ui_root()
        .spawn_scene(("ui/hello.cobweb", "scene"), &mut scene_builder, |h| {
            h.insert_reactive(SeedSelection::default());
            let scene_entity = h.id();

            h.edit("seeds::seed_button_daisy", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Daisy);
                        c.react().broadcast(CurrentSeedUpdate);
                        OK
                    },
                );
            });
            h.edit("seeds::seed_button_pineapple", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Pineapple);
                        c.react().broadcast(CurrentSeedUpdate);
                        OK
                    },
                );
            });
            h.edit("seeds::seed_button_dragonfruit", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Dragonfruit);
                        c.react().broadcast(CurrentSeedUpdate);
                        OK
                    },
                );
            });
            h.edit("seeds::seed_button_gnome", |h| {
                h.on_pressed(
                    move |mut c: Commands, mut seed_selection: ReactiveMut<SeedSelection>| {
                        seed_selection
                            .get_mut(&mut c, scene_entity)?
                            .set_seed_type(PlantType::Gnome);
                        c.react().broadcast(CurrentSeedUpdate);
                        OK
                    },
                );
            });

            h.edit("reset_button", |h| {
                h.on_pressed(
                    move |mut restart_game_events: EventWriter<RestartGameEvent>| {
                        restart_game_events.write_default();
                        OK
                    },
                );
            });

            h.get("current_seed").update_on(
                broadcast::<CurrentSeedUpdate>(),
                move |id: TargetId,
                      mut editor: TextEditor,
                      q_seed_selection: Reactive<SeedSelection>| {
                    let (_, seed_selection) = q_seed_selection.single();
                    let seed_type = seed_selection.seed_type();
                    write_text!(editor, *id, "Current seed: {:?}", seed_type);
                    info!("Update UI for seed type {:?}", seed_type);
                },
            );

            h.get("bank").update_on(
                broadcast::<BalanceUpdate>(),
                move |id: TargetId, mut editor: TextEditor, q_bank_account: Query<&BankAccount>| {
                    let Ok(bank_account) = q_bank_account.single() else {
                        warn!("No bank account");
                        return;
                    };
                    let balance = bank_account.balance();
                    write_text!(
                        editor,
                        *id,
                        "Bank balance: ${}/${}",
                        balance,
                        WINNING_BALANCE
                    );
                    info!("Update UI for bank account balance {:?}", balance);
                },
            );
        });
}

fn update_ui(
    mut commands: Commands,
    mut bank_account_update_events: EventReader<BankAccountUpdateEvent>,
) {
    for _ in bank_account_update_events.read() {
        commands.react().broadcast(BalanceUpdate);
    }
}
