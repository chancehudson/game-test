use macroquad::prelude::*;

mod game;

use game::GameState;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut game = GameState::new();
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    loop {
        clear_background(RED);
        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text(&format!("fps: {fps}"), 0., 20., 19., BLACK);
        // game will handle setting the appropriate camera
        game.render();

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        draw_text("IT WORKS!", 20.0, 80.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
