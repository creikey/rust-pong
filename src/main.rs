use raylib::prelude::*;

mod imui;
mod awaiting_opponent;
mod pong;
mod title_screen;

pub const SCREEN_SIZE: Vector2 = Vector2::new(1000.0, 800.0);

pub struct SceneAPI {
    pub new_scene: Option<Box<dyn Scene>>,
}

pub trait Scene {
    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle);
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle);

    fn should_quit(&self) -> bool;
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_SIZE.x as i32, SCREEN_SIZE.y as i32)
        .title("Rust Pong")
        .build();

    let mut cur_scene: Box<dyn Scene> = Box::new(title_screen::TitleScreen::new());
    let mut scene_api = SceneAPI { new_scene: None };

    while !rl.window_should_close() {
        cur_scene.process(&mut scene_api, &mut rl);

        let mut d = rl.begin_drawing(&thread);

        cur_scene.draw(&mut scene_api, &mut d);

        if cur_scene.should_quit() {
            break;
        }

        if scene_api.new_scene.is_some() {
            cur_scene = scene_api.new_scene.unwrap();
            scene_api.new_scene = None;
        }
    }
}
