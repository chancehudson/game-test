use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_egui::egui;
use bevy_egui::egui::Color32;
use bevy_egui::egui::CornerRadius;
use bevy_egui::egui::Margin;
use bevy_egui::egui::Pos2;
use bevy_egui::egui::RichText;
use bevy_egui::egui::Stroke;
use bevy_egui::egui::StrokeKind;
use bevy_egui::egui::Vec2;

use crate::GameState;
use crate::plugins::engine::ActivePlayerState;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            show_bottom_info_bar
                .run_if(in_state(GameState::OnMap).or(in_state(GameState::LoadingMap))),
        );
    }
}

fn show_bottom_info_bar(mut contexts: EguiContexts, active_player: Res<ActivePlayerState>) {
    if active_player.0.is_none() {
        return;
    }
    let active_player = &active_player.0.as_ref().unwrap();
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
                    ui.label("lvl. 5");
                });
                ui.vertical(|ui| {
                    ui.horizontal_top(|ui| {
                        // health bar
                        bar(ui, 50., 100., Color32::RED, 80., 20.);
                        // mana bar
                        bar(ui, 100., 100., Color32::BLUE, 80., 20.);
                    });
                    ui.horizontal_top(|ui| {
                        bar(ui, 10., 1000., Color32::DARK_GREEN, 160., 20.);
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

fn bar(ui: &mut egui::Ui, current: f32, max: f32, color: Color32, width: f32, height: f32) {
    let corner_radius = 4 as u8;
    let percentage = (current / max).clamp(0.0, 1.0);
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
