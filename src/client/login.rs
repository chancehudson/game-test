use macroquad::prelude::*;
use macroquad::ui::hash;
use macroquad::ui::root_ui;
use macroquad::ui::widgets;

pub struct LoginScreen {
    pub username: String,
    pub password: String,
    pub error_message: Option<String>,
}

impl LoginScreen {
    pub fn new() -> Self {
        root_ui().set_input_focus(99);
        Self {
            username: String::new(),
            password: String::new(),
            error_message: None,
        }
    }

    pub fn draw(&mut self) -> (bool, bool) {
        let mut login_clicked = false;
        let mut create_clicked = false;
        let size = vec2(300., 300.);
        let pos = vec2(
            (screen_width() - size.x) / 2.,
            (screen_height() - size.y) / 2.,
        );
        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));
        widgets::Window::new(hash!(), pos, size)
            .label("login")
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, "Welcome");

                ui.separator();
                ui.input_text(99, "username", &mut self.username);
                ui.input_password(hash!(), "password", &mut self.password);
                ui.separator();
                if ui.button(None, "submit") || is_key_pressed(KeyCode::Enter) {
                    login_clicked = true;
                }
                ui.same_line(0.);
                if ui.button(None, "create") {
                    create_clicked = true;
                }
                if let Some(err) = &self.error_message {
                    ui.separator();
                    ui.label(None, err);
                }
            });
        (login_clicked, create_clicked)
    }
}
