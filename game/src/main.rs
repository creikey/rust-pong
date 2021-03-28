use std::env;

// scenes - these effectively act as separate games
pub mod awaiting_opponent;
pub mod pong; // pong game logic, ui, and rollback networking
pub mod title_screen; // title screen buttons and scene switching logic // screen that polls the server waiting for an opponent to join

// utility functions - these are more like libraries
pub mod imui;
pub mod scene; // scene API and scene struct/trait // immediate mode ui

use scene::*;

fn main() {
    let window_width = pong::GAME_CONFIG.arena_size.x as i32;

    let (mut rl, thread) = raylib::init()
        .size(window_width, pong::GAME_CONFIG.arena_size.y as i32)
        .title("Rust Pong")
        .build();

    // easier development forcing the window to go to the right or the left
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "left" {
            rl.set_window_position(0, rl.get_window_position().y as i32);
        } else if args[1] == "right" {
            rl.set_window_position(
                rl.get_screen_width(), // - window_width,
                rl.get_window_position().y as i32,
            );
        } else {
            println!("unknown argument {}", args[1]);
        }
    }

    rl.set_target_fps(60);

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
