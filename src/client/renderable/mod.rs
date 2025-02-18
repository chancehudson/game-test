mod mob;

pub use mob::MobRenderable;

pub trait Renderable {
    fn render(&mut self, step_len: f32);
}
