use raylib::prelude::*;

mod pong;

pub const SCREEN_SIZE: Vector2 = Vector2::new(1000.0, 800.0);

fn rect_new_ex(position: Vector2, size: Vector2) -> Rectangle {
    Rectangle::new(position.x, position.y, size.x, size.y)
}

fn rect_pos(r: &Rectangle) -> Vector2 {
    Vector2::new(r.x, r.y)
}

fn rect_size(r: &Rectangle) -> Vector2 {
    Vector2::new(r.width, r.height)
}

/* tabling for now to use raylib rectangle
struct Rectangle {
    upper_left_corner: Vector2,
    size: Vector2,
}

impl Rectangle {
    fn has_point(&self, point: Vector2) -> bool {
        point.x >= self.upper_left_corner.x
            && point.x <= self.upper_left_corner.x + self.size.x
            && point.y >= self.upper_left_corner.y
            && point.y <= self.upper_left_corner.y + self.size.y
    }
}*/

struct TitleScreen {
    play_pong_game: bool,
}

impl TitleScreen {
    fn new() -> Self {
        TitleScreen {
            play_pong_game: false,
        }
    }
}

impl Scene for TitleScreen {
    fn process(&mut self, rl: &RaylibHandle) {}
    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::GRAY);

        let screen_size = Vector2::new(d.get_screen_width() as f32, d.get_screen_height() as f32);
        let font_size = 50.0;
        let size = measure_text_ex(d.get_font_default(), "PLAY", font_size, 1.0);
        let bounding_box = rect_new_ex(screen_size / 2.0 - size / 2.0, size);
        let hovered = bounding_box.check_collision_point_rec(d.get_mouse_position());

        let background_color = if hovered {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(220, 220, 220, 255)
        };

        d.draw_rectangle_v(
            rect_pos(&bounding_box),
            rect_size(&bounding_box),
            background_color,
        );
        d.draw_text_ex(
            d.get_font_default(),
            "PLAY",
            Vector2::new(bounding_box.x, bounding_box.y),
            font_size,
            1.0,
            Color::BLACK,
        );

        if d.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) && hovered {
            self.play_pong_game = true;
        }
    }
    fn get_new_scene(&self) -> Option<Box<dyn Scene>> {
        if self.play_pong_game {
            Some(Box::new(pong::PongGame::new()))
        } else {
            None
        }
    }
}

pub trait Scene {
    fn process(&mut self, rl: &RaylibHandle);
    fn draw(&mut self, d: &mut RaylibDrawHandle);
    fn get_new_scene(&self) -> Option<Box<dyn Scene>>;
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_SIZE.x as i32, SCREEN_SIZE.y as i32)
        .title("Rust Pong")
        .build();

    let mut cur_scene: Box<dyn Scene> = Box::new(TitleScreen::new());

    while !rl.window_should_close() {
        cur_scene.process(&rl);

        let mut d = rl.begin_drawing(&thread);

        cur_scene.draw(&mut d);

        let new_scene = cur_scene.get_new_scene();
        if new_scene.is_some() {
            cur_scene = new_scene.unwrap();
        }
    }
}
