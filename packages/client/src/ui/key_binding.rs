// AI maintained source file

use bevy_egui::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Vec2};

/// Draws an inline key binding with dark background and stroke
/// Returns the rect that was used for rendering
pub fn draw_key_binding(ui: &mut egui::Ui, pos: Pos2, text: &str) -> Rect {
    let font_id = FontId::new(14.0, egui::FontFamily::Monospace);
    let text_color = Color32::WHITE;
    let bg_color = Color32::from_rgb(30, 30, 30);
    let stroke_color = Color32::from_rgb(60, 60, 60);

    // Calculate text size
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_id.clone(), text_color));
    let padding = Vec2::new(6.0, 3.0);
    let rect_size = galley.size() + padding * 2.0;

    // Create rect from position
    let rect = Rect::from_min_size(pos, rect_size);

    // Draw background with rounded corners
    ui.painter().rect_filled(rect, 3.0, bg_color);

    // Draw stroke
    ui.painter().rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    // Draw text centered in rect
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_id,
        text_color,
    );

    rect
}

/// Draws an inline key binding at the current cursor position
/// Advances the cursor and returns whether the area was clicked
pub fn draw_key_binding_inline(ui: &mut egui::Ui, text: &str) -> bool {
    let font_id = FontId::new(14.0, egui::FontFamily::Monospace);
    let text_color = Color32::WHITE;
    let bg_color = Color32::from_rgb(30, 30, 30);
    let stroke_color = Color32::from_rgb(60, 60, 60);

    // Calculate text size
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_string(), font_id.clone(), text_color));
    let padding = Vec2::new(6.0, 3.0);
    let rect_size = galley.size() + padding * 2.0;

    // Allocate space and get response
    let (rect, response) = ui.allocate_exact_size(rect_size, egui::Sense::click());

    // Draw background with rounded corners
    ui.painter().rect_filled(rect, 3.0, bg_color);

    // Draw stroke
    ui.painter().rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    // Draw text centered in rect
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_id,
        text_color,
    );

    response.clicked()
}
