use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy_egui::egui::Align2;
use bevy_egui::egui::Color32;
use bevy_egui::egui::FontId;
use bevy_egui::egui::Id;
use bevy_egui::egui::Pos2;
use bevy_egui::egui::Rect;
use bevy_egui::egui::ScrollArea;

use db::PlayerInventory;
use game_common::data::GameData;
use game_common::network::Action;
use game_common::network::Response;

use crate::GameState;
use crate::network::NetworkAction;
use crate::network::NetworkMessage;
use crate::plugins::database::Database;
use crate::plugins::game_data_loader::GameDataResource;
use crate::sprite_data_loader::SpriteManager;

#[derive(Resource, Default)]
pub struct PlayerInventoryRes(pub PlayerInventory);

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PlayerInventoryState {
    #[default]
    Hidden,
    Visible,
}

#[derive(Resource, Default)]
pub struct PlayerInventoryGuiData {
    dragging_entry: (u8, (u64, u32)),
    dragging_index: u8, // the inventory slot index
    is_dragging: bool,
}

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInventoryRes>()
            .init_resource::<PlayerInventoryGuiData>()
            .init_state::<PlayerInventoryState>()
            .add_systems(
                Update,
                show_inventory_gui.run_if(
                    in_state(GameState::OnMap).and(in_state(PlayerInventoryState::Visible)),
                ),
            )
            .add_systems(FixedUpdate, handle_player_inventory)
            .add_systems(OnEnter(GameState::LoggedOut), reset_inventory);
    }
}

fn reset_inventory(mut commands: Commands) {
    commands.insert_resource(Database::default());
}

fn handle_player_inventory(
    db: Res<Database>,
    mut action_events: EventReader<NetworkMessage>,
    mut player_inventory: ResMut<PlayerInventoryRes>,
) -> Result<()> {
    for event in action_events.read() {
        if let Response::PlayerInventoryRecord(slot_index, entry) = &event.0 {
            let db = db.0.clone();
            player_inventory.0.insert(db, *slot_index, *entry)?;
        }
    }
    Ok(())
}

fn draw_drag_window(
    rect: Rect,
    contexts: &mut EguiContexts,
    game_data: &GameData,
    sprite_manager: &mut ResMut<SpriteManager>,
    asset_server: &Res<AssetServer>,
    inventory_gui_data: &mut ResMut<PlayerInventoryGuiData>,
) {
    if !inventory_gui_data.is_dragging {
        return;
    }
    egui::Area::new(Id::new("item_drag"))
        .interactable(false)
        .fixed_pos(rect.min)
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            ui.allocate_exact_size(rect.size(), egui::Sense::empty());
            #[cfg(debug_assertions)]
            assert_ne!(inventory_gui_data.dragging_entry.1.0, 0);
            #[cfg(debug_assertions)]
            assert_ne!(inventory_gui_data.dragging_entry.1.1, 0);
            if let Some(item) = game_data.items.get(&inventory_gui_data.dragging_entry.1.0) {
                if !sprite_manager.is_animation_loaded(&item.icon_animation, &asset_server) {
                    sprite_manager.load_animation(&item.icon_animation);
                    return;
                }
                if let Some(egui_texture) =
                    sprite_manager.egui_handle(&item.icon_animation.sprite_sheet)
                {
                    ui.put(
                        rect,
                        egui::Image::new(egui_texture)
                            .fit_to_exact_size(egui::Vec2::new(48.0, 48.0)),
                    );
                }
            } else {
                println!(
                    "Item with unknown id: {} {:?}",
                    inventory_gui_data.dragging_entry.1.0, game_data.items
                );
                unreachable!();
            }
        });
}

fn show_inventory_gui(
    mut contexts: EguiContexts,
    mut player_inventory: ResMut<PlayerInventoryRes>,
    database: Res<Database>,
    game_data: Res<GameDataResource>,
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
    mut inventory_gui_data: ResMut<PlayerInventoryGuiData>,
    mut action_events: EventWriter<NetworkAction>,
) {
    let game_data = &game_data.0;
    let item_size = egui::Vec2::new(50.0, 50.0);
    let pointer_pos = contexts.ctx_mut().pointer_latest_pos().unwrap_or_default();
    draw_drag_window(
        Rect::from_min_size(pointer_pos + egui::Vec2::splat(3.), item_size),
        &mut contexts,
        &game_data,
        &mut sprite_manager,
        &asset_server,
        &mut inventory_gui_data,
    );
    egui::Window::new("Inventory")
        .default_height(300.)
        .min_width(150.)
        .max_width(150.)
        .default_pos([300., 100.])
        .show(contexts.ctx_mut(), |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show_viewport(ui, |ui, _viewport| {
                    let is_dragging_last_frame = inventory_gui_data.is_dragging;
                    if inventory_gui_data.is_dragging && !ui.input(|i| i.pointer.primary_down()) {
                        inventory_gui_data.is_dragging = false;
                        let mouse_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();
                        if !ui.clip_rect().contains(mouse_pos) {
                            // it's a drop
                            player_inventory
                                .0
                                .drop(
                                    database.0.clone(),
                                    inventory_gui_data.dragging_entry.0,
                                    inventory_gui_data.dragging_entry.1.1,
                                )
                                .unwrap();
                            action_events.write(NetworkAction(Action::PlayerInventoryDrop(
                                inventory_gui_data.dragging_entry.0,
                                inventory_gui_data.dragging_entry.1.1,
                            )));
                        }
                    }
                    let items_per_row = 5usize;

                    for row in 0..=(u8::MAX as usize / items_per_row) {
                        ui.horizontal(|ui| {
                            for col in 0..items_per_row {
                                let i = (row * items_per_row) + col;
                                if i > u8::MAX as usize {
                                    break;
                                }
                                let i = i as u8;
                                let (rect, response) = ui
                                    .allocate_exact_size(item_size, egui::Sense::click_and_drag());

                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                                    egui::epaint::StrokeKind::Inside, // or Inside, Center
                                );

                                let entry_maybe =
                                    player_inventory.0.items.get(&i).map(|v| v.clone());
                                if response.drag_started()
                                    && let Some(entry) = entry_maybe
                                {
                                    inventory_gui_data.dragging_entry = (i, entry);
                                    inventory_gui_data.is_dragging = true;
                                    inventory_gui_data.dragging_index = i;
                                }

                                // Stop dragging when mouse released
                                if is_dragging_last_frame && !ui.input(|i| i.pointer.primary_down())
                                {
                                    let mouse_pos =
                                        ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();
                                    if rect.contains(mouse_pos)
                                        && inventory_gui_data.dragging_entry.0 != i
                                    {
                                        // atomic swap slot contents
                                        player_inventory
                                            .0
                                            .swap(
                                                database.0.clone(),
                                                (inventory_gui_data.dragging_entry.0, i),
                                            )
                                            .unwrap();
                                        action_events.write(NetworkAction(
                                            Action::PlayerInventorySwap((
                                                inventory_gui_data.dragging_entry.0,
                                                i,
                                            )),
                                        ));
                                    }
                                }
                                // render icon, only if not being dragged
                                if let Some((item_type, count)) = entry_maybe
                                    && let Some(item) = game_data.items.get(&item_type)
                                    && (!is_dragging_last_frame
                                        || inventory_gui_data.dragging_entry.0 != i)
                                {
                                    if !sprite_manager
                                        .is_animation_loaded(&item.icon_animation, &asset_server)
                                    {
                                        sprite_manager.load_animation(&item.icon_animation);
                                        continue;
                                    }
                                    if let Some(egui_texture) = sprite_manager
                                        .egui_handle(&item.icon_animation.sprite_sheet)
                                    {
                                        ui.put(
                                            rect,
                                            egui::Image::new(egui_texture)
                                                .fit_to_exact_size(egui::Vec2::new(48.0, 488.0)),
                                        );
                                        ui.painter().text(
                                            rect.left_bottom(),
                                            Align2::LEFT_BOTTOM,
                                            format!("{count}"),
                                            FontId::monospace(14.0),
                                            Color32::WHITE,
                                        );
                                    }
                                }
                            }
                        });
                    }
                });
        });
}
