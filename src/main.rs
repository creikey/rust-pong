use raylib::prelude::*;

mod pong;
mod title_screen;

pub const SCREEN_SIZE: Vector2 = Vector2::new(1000.0, 800.0);

pub trait Scene {
    fn process(&mut self, rl: &RaylibHandle);
    fn draw(&mut self, d: &mut RaylibDrawHandle);

    fn get_new_scene(&self) -> Option<Box<dyn Scene>>;
    fn should_quit(&self) -> bool;
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_SIZE.x as i32, SCREEN_SIZE.y as i32)
        .title("Rust Pong")
        .build();

    let mut cur_scene: Box<dyn Scene> = Box::new(title_screen::TitleScreen::new());

    while !rl.window_should_close() {
        cur_scene.process(&rl);

        let mut d = rl.begin_drawing(&thread);

        cur_scene.draw(&mut d);

        if cur_scene.should_quit() {
            break;
        }

        let new_scene = cur_scene.get_new_scene();
        if new_scene.is_some() {
            cur_scene = new_scene.unwrap();
        }
    }
}
