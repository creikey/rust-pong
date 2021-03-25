use rust_pong::scene::*;
use rust_pong::pong;
use rust_pong::title_screen;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(pong::GAME_CONFIG.arena_size.x as i32, pong::GAME_CONFIG.arena_size.y as i32)
        .title("Rust Pong")
        .build();

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
