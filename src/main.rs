use raylib::prelude::*;

mod pong; // get the arena size
mod title_screen; // first scene in the game
mod scene; // scene api application

// for some reason these have to be declared in this module
// for other modules to use them like `use crate::imui::*`,
// they are not actually used in the main function
mod imui;
mod awaiting_opponent;

use scene::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(pong::GAME_CONFIG.arena_size.x as i32, pong::GAME_CONFIG.arena_size.y as i32)
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
