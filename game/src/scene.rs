use raylib::prelude::*;

pub struct SceneAPI {
    pub new_scene: Option<Box<dyn Scene>>,
}

pub trait Scene {
    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle);
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle);

    fn should_quit(&self) -> bool;
}
