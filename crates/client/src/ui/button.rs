// AI maintained source file

use bevy_egui::egui;
use egui::Align2;
use egui::Color32;
use egui::CornerRadius;
use egui::FontId;
use egui::Pos2;
use egui::Rect;
use egui::Vec2;

// Reusable button function
pub fn draw_root_button(
    ui: &mut egui::Ui,
    rect: Rect,
    text: &str,
    hotkey: Option<&str>,
    is_active: bool,
) -> bool {
    // Get cursor position and mouse state
    let cursor_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
    let is_hovered = rect.contains(cursor_pos);
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());

    // Button was activated if it's hovered and mouse was clicked this frame
    let button_activated = is_hovered && mouse_clicked;

    // Visual state for rendering
    let is_pressed = is_hovered && mouse_down;

    // Button colors based on state
    let (bg_color_top, bg_color_bottom, border_color) = if is_active {
        (
            Color32::from_rgb(200, 160, 60),
            Color32::from_rgb(150, 120, 20),
            Color32::from_rgb(120, 90, 10),
        )
    } else if is_pressed {
        (
            Color32::from_rgb(60, 120, 180),
            Color32::from_rgb(40, 90, 140),
            Color32::from_rgb(30, 70, 110),
        )
    } else if is_hovered {
        (
            Color32::from_rgb(80, 140, 200),
            Color32::from_rgb(60, 120, 180),
            Color32::from_rgb(50, 100, 160),
        )
    } else {
        (
            Color32::from_rgb(70, 130, 190),
            Color32::from_rgb(50, 110, 170),
            Color32::from_rgb(40, 90, 150),
        )
    };

    // Adjust rect for pressed effect
    let button_rect = if is_pressed {
        Rect::from_min_size(
            rect.min + Vec2::new(1., 1.),
            rect.size() - Vec2::new(2., 2.),
        )
    } else {
        rect
    };

    // Draw shadow
    ui.painter().rect_filled(
        Rect::from_min_size(button_rect.min + Vec2::new(2., 2.), button_rect.size()),
        CornerRadius::same(8),
        Color32::from_rgba_premultiplied(0, 0, 0, 60),
    );

    // Draw gradient background
    draw_gradient_rect(ui, button_rect.shrink(2.0), bg_color_top, bg_color_bottom);

    // Draw border and highlight
    ui.painter().rect_stroke(
        button_rect,
        CornerRadius::same(8),
        egui::Stroke::new(2.0, border_color),
        egui::StrokeKind::Inside,
    );
    ui.painter().rect_stroke(
        button_rect.shrink(3.0),
        CornerRadius::same(6),
        egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 100)),
        egui::StrokeKind::Inside,
    );

    // Draw text with shadow/outline for better visibility
    let center = button_rect.center();
    let text_offset = if hotkey.is_some() { -8.0 } else { 0.0 };

    // Multiple shadow layers for stronger outline effect
    for offset in [
        Vec2::new(-1., -1.),
        Vec2::new(1., -1.),
        Vec2::new(-1., 1.),
        Vec2::new(1., 1.),
        Vec2::new(0., -1.),
        Vec2::new(0., 1.),
        Vec2::new(-1., 0.),
        Vec2::new(1., 0.),
    ] {
        ui.painter().text(
            center + Vec2::new(0., text_offset) + offset,
            Align2::CENTER_CENTER,
            text,
            FontId::new(14.0, egui::FontFamily::Proportional),
            Color32::BLACK,
        );

        if let Some(hotkey_text) = hotkey {
            ui.painter().text(
                center + Vec2::new(0., 8.) + offset,
                Align2::CENTER_CENTER,
                hotkey_text,
                FontId::new(12.0, egui::FontFamily::Proportional),
                Color32::BLACK,
            );
        }
    }

    // Main text in bold white
    ui.painter().text(
        center + Vec2::new(0., text_offset),
        Align2::CENTER_CENTER,
        text,
        FontId::new(14.0, egui::FontFamily::Proportional),
        Color32::WHITE,
    );

    if let Some(hotkey_text) = hotkey {
        ui.painter().text(
            center + Vec2::new(0., 8.),
            Align2::CENTER_CENTER,
            hotkey_text,
            FontId::new(12.0, egui::FontFamily::Proportional),
            Color32::WHITE,
        );
    }

    // Return whether the button was activated this frame
    button_activated
}

// Helper function to draw gradient background
fn draw_gradient_rect(ui: &mut egui::Ui, rect: Rect, top_color: Color32, bottom_color: Color32) {
    let painter = ui.painter();
    let steps = 10;
    let step_height = rect.height() / steps as f32;

    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let color = Color32::from_rgb(
            (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8,
            (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8,
            (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8,
        );

        let y = rect.min.y + i as f32 * step_height;
        let line_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, y),
            Vec2::new(rect.width(), step_height + 1.0),
        );
        painter.rect_filled(line_rect, CornerRadius::same(6), color);
    }
}
