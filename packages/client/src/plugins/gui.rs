use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy_egui::egui::Color32;
use bevy_egui::egui::CornerRadius;
use bevy_egui::egui::Id;
use bevy_egui::egui::Margin;
use bevy_egui::egui::Pos2;
use bevy_egui::egui::Rect;
use bevy_egui::egui::RichText;
use bevy_egui::egui::Stroke;
use bevy_egui::egui::StrokeKind;
use bevy_egui::egui::Vec2;
use egui::Align2;
use egui::FontId;

use db::Ability;
use game_common::entity::EngineEntity;

use crate::GameState;
use crate::plugins::engine::ActiveGameEngine;
use crate::plugins::engine::ActivePlayerEntityId;
use crate::plugins::engine::ActivePlayerState;
use crate::plugins::player_inventory::PlayerInventoryState;
use crate::ui::draw_root_button;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (show_bottom_info_bar, show_bottom_buttons)
                .run_if(in_state(GameState::OnMap).or(in_state(GameState::LoadingMap))),
        );
    }
}

fn show_bottom_buttons(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<PlayerInventoryState>>,
    state: Res<State<PlayerInventoryState>>,
) {
    let screen_rect = contexts.ctx_mut().screen_rect();
    let size = Vec2::new(80., 50.);
    let min = screen_rect.min
        + Vec2::new(
            screen_rect.max.x - 2.0 * size.x,
            screen_rect.max.y - (5.0 + size.y),
        );

    egui::Area::new(Id::new("inventory_bottom_button"))
        .movable(false)
        .show(contexts.ctx_mut(), |ui| {
            let rect = Rect::from_min_size(min, size);
            let is_inventory_open = matches!(state.get(), PlayerInventoryState::Visible);

            let response = draw_root_button(ui, rect, "Inventory", Some("(I)"), is_inventory_open);

            if response.clicked() {
                match state.get() {
                    PlayerInventoryState::Visible => next_state.set(PlayerInventoryState::Hidden),
                    PlayerInventoryState::Hidden => next_state.set(PlayerInventoryState::Visible),
                }
            }
        });
}

fn show_bottom_info_bar(
    mut contexts: EguiContexts,
    active_player: Res<ActivePlayerState>,
    active_engine: Res<ActiveGameEngine>,
    active_player_entity_id: Res<ActivePlayerEntityId>,
) {
    if active_player.0.is_none() || active_player_entity_id.0.is_none() {
        return;
    }
    let entity_id = active_player_entity_id.0.unwrap();
    let player_entity = active_engine.0.entities.get(&entity_id);
    if player_entity.is_none() {
        return;
    }
    let player_entity = match player_entity.unwrap() {
        EngineEntity::Player(p) => p,
        _ => unreachable!(),
    };
    let active_player = &active_player.0.as_ref().unwrap();
    let player_level = player_entity.stats.total_level();
    let health_level = player_entity.stats.next_level(&Ability::Health);
    let strength_level = player_entity.stats.next_level(&Ability::Strength);
    let dex_level = player_entity.stats.next_level(&Ability::Dexterity);
    let int_level = player_entity.stats.next_level(&Ability::Intelligence);
    egui::Window::new("bottom_info_bar")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, 0.0))
        .frame(egui::Frame {
            fill: Color32::DARK_GRAY,
            inner_margin: Margin::same(10),
            corner_radius: CornerRadius::same(4),
            ..Default::default()
        })
        .show(contexts.ctx_mut(), |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::LIGHT_GRAY);
            // ui.allocate_space(ui.available_size());
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&active_player.username).size(18.0));
                    ui.label(format!("lvl. {}", player_level));
                });
                ui.vertical(|ui| {
                    ui.horizontal_top(|ui| {
                        // health bar
                        bar(
                            ui,
                            player_entity.record.current_health,
                            player_entity.stats.max_health(),
                            Color32::RED,
                            80.,
                            20.,
                        );
                        // mana bar
                        bar(
                            ui,
                            100,
                            100,
                            // player_entity.stats.current_mana,
                            // player_entity.stats.max_mana,
                            Color32::BLUE,
                            80.,
                            20.,
                        );
                    });

                    // This needs to be a different expereinece bar depending on what experience the user last received.
                    // So if the user is receving strength experience it should switch to indicate that
                    //
                    // we'll hardcode it for now
                    ui.vertical(|ui| {
                        small_bar(ui, health_level, Color32::DARK_RED, 160., 10., "health");
                        small_bar(ui, dex_level, Color32::LIGHT_BLUE, 160., 10., "dex");
                        small_bar(
                            ui,
                            strength_level,
                            Color32::LIGHT_RED,
                            160.,
                            10.,
                            "strength",
                        );
                        small_bar(ui, int_level, Color32::DARK_BLUE, 160., 10., "int");
                    });
                });
                ui.vertical(|ui| {
                    if ui.button("Logout").clicked() {
                        println!("logout clicked");
                    }
                });
            });
        });
}

fn small_bar(
    ui: &mut egui::Ui,
    (percent, current, max, level): (f64, u64, u64, u64),
    color: Color32,
    width: f32,
    height: f32,
    name: &str,
) {
    let _current = current as f64;
    let _max = max as f64;
    let corner_radius = 4 as u8;
    let fill_width = width * percent as f32;

    // Always allocate the full width to maintain layout
    let (full_rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

    // Background frame (always full rounded)
    ui.painter().rect_filled(
        full_rect,
        corner_radius as f32,
        Color32::DARK_GRAY, // background color
    );

    if percent > 0.0 {
        // Fill frame with conditional rounding
        let fill_rect = egui::Rect::from_min_size(full_rect.min, Vec2::new(fill_width, height));

        let rounding = if percent >= 1.0 {
            egui::CornerRadius::same(corner_radius) // All corners rounded when full
        } else {
            egui::CornerRadius {
                nw: corner_radius, // top-left rounded
                ne: 0,             // top-right square
                sw: corner_radius, // bottom-left rounded
                se: 0,             // bottom-right square
            }
        };

        ui.painter().rect_filled(fill_rect, rounding, color);
    }

    ui.painter().rect_stroke(
        full_rect,
        corner_radius as f32,
        Stroke {
            width: 1.0,
            color: Color32::BLACK,
        },
        StrokeKind::Inside,
    );
    let end_color = Color32::from_black_alpha(140);
    ui.painter().rect_filled(
        Rect::from_min_max(
            full_rect.min,
            Pos2::new(full_rect.min.x + 20.0, full_rect.max.y),
        ),
        egui::CornerRadius {
            nw: corner_radius,
            ne: 0,
            sw: corner_radius,
            se: 0,
        },
        end_color,
    );
    ui.painter().rect_filled(
        Rect::from_min_max(
            Pos2::new(full_rect.max.x - 60.0, full_rect.min.y),
            full_rect.max,
        ),
        egui::CornerRadius {
            nw: 0,
            ne: corner_radius,
            sw: 0,
            se: corner_radius,
        },
        end_color,
    );
    // Text overlay (always centered on full width)
    ui.painter().text(
        Pos2::new(full_rect.left() + 8., full_rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{:.000} %", 100. * percent),
        egui::FontId::proportional(8.0),
        Color32::WHITE,
    );
    ui.painter().text(
        Pos2::new(full_rect.right() - 8., full_rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format!("{} lvl {}", name, level),
        egui::FontId::proportional(8.0),
        Color32::WHITE,
    );
}

fn bar(ui: &mut egui::Ui, current: u64, max: u64, color: Color32, width: f32, height: f32) {
    let current = current as f64;
    let max = max as f64;
    let corner_radius = 4 as u8;
    let percentage = (current / max).clamp(0.0, 1.0) as f32;
    let fill_width = width * percentage;

    // Always allocate the full width to maintain layout
    let (full_rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

    // Background frame (always full rounded)
    ui.painter().rect_filled(
        full_rect,
        corner_radius as f32,
        Color32::DARK_GRAY, // background color
    );

    if percentage > 0.0 {
        // Fill frame with conditional rounding
        let fill_rect = egui::Rect::from_min_size(full_rect.min, Vec2::new(fill_width, height));

        let rounding = if percentage >= 1.0 {
            egui::CornerRadius::same(corner_radius) // All corners rounded when full
        } else {
            egui::CornerRadius {
                nw: corner_radius, // top-left rounded
                ne: 0,             // top-right square
                sw: corner_radius, // bottom-left rounded
                se: 0,             // bottom-right square
            }
        };

        ui.painter().rect_filled(fill_rect, rounding, color);
    }

    ui.painter().rect_stroke(
        full_rect,
        corner_radius as f32,
        Stroke {
            width: 1.0,
            color: Color32::BLACK,
        },
        StrokeKind::Inside,
    );
    // Text overlay (always centered on full width)
    ui.painter().text(
        full_rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.0} / {:.0}", current, max),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}
