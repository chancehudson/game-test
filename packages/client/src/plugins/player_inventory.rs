use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy_egui::egui::Align2;
use bevy_egui::egui::Color32;
use bevy_egui::egui::CornerRadius;
use bevy_egui::egui::FontId;
use bevy_egui::egui::Pos2;
use bevy_egui::egui::Rect;
use bevy_egui::egui::ScrollArea;

use db::PlayerInventory;
use game_common::data::GameData;
use game_common::network::Response;

use crate::GameState;
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
    position: Pos2,
    dragging_index: u8,
    is_dragging: bool,
    drag_start_pos: Pos2,
    drag_current_pos: Pos2,
}

impl PlayerInventoryGuiData {
    pub fn offset(&self) -> Pos2 {
        self.position + (self.drag_current_pos - self.drag_start_pos)
    }
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
    egui::Window::new("item_drag")
        .fixed_rect(rect)
        .title_bar(false)
        .collapsible(false)
        .interactable(false)
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT, // Transparent background
            corner_radius: CornerRadius::ZERO,
            ..Default::default()
        })
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
    player_inventory: Res<PlayerInventoryRes>,
    game_data: Res<GameDataResource>,
    mut sprite_manager: ResMut<SpriteManager>,
    asset_server: Res<AssetServer>,
    mut inventory_gui_data: ResMut<PlayerInventoryGuiData>,
) {
    let game_data = &game_data.0;
    let item_size = egui::Vec2::new(50.0, 50.0);
    draw_drag_window(
        Rect::from_min_size(inventory_gui_data.offset(), item_size),
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
                    let items_per_row = 8usize;

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

                                if player_inventory.0.items.get(&i).is_none() {
                                    // disallow interaction if slot is empty
                                    continue;
                                }
                                let (item_type, count) = player_inventory.0.items.get(&i).unwrap();
                                if response.drag_started() {
                                    inventory_gui_data.dragging_entry = (i, (*item_type, *count));
                                    inventory_gui_data.position = rect.min;
                                    inventory_gui_data.is_dragging = true;
                                    inventory_gui_data.dragging_index = i;
                                    inventory_gui_data.drag_start_pos =
                                        response.interact_pointer_pos().unwrap_or_default();
                                }
                                if inventory_gui_data.is_dragging {
                                    // Use global input instead of response.dragged()
                                    if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                                        inventory_gui_data.drag_current_pos = pointer_pos;
                                    }

                                    // Stop dragging when mouse released
                                    if !ui.input(|i| i.pointer.primary_down()) {
                                        inventory_gui_data.is_dragging = false;
                                    }
                                }
                                if let Some(item) = game_data.items.get(item_type) {
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
                                } else {
                                    println!(
                                        "Item with unknown id: {} {:?}",
                                        item_type, game_data.items
                                    );
                                    unreachable!();
                                }
                            }
                        });
                    }
                });
        });
}
