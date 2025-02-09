use macroquad::prelude::*;

mod game;
mod player;
mod map;

use game::GameState;
pub use game::Actor;
pub use player::Player;
pub use map::Map;

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut game = GameState::new().await;
    let fps_render_interval = 1.0;
    let mut next_fps_render = 0.0;
    let mut fps = 0.0;
    loop {
        clear_background(RED);
        // game will handle setting the appropriate camera
        game.render();

        set_default_camera();
        // render ui components
        if get_time() > next_fps_render {
            next_fps_render = get_time() + fps_render_interval;
            fps = get_fps().into();
        }
        draw_text(&format!("fps: {fps}"), 0., 20., 19., BLACK);
        next_frame().await
    }
}
