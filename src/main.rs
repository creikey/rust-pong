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


fn button(d: &mut RaylibDrawHandle, upper_left_corner: Vector2, size: Vector2, text: &str) -> bool {
    let font_size = 50.0;
    let bounding_box = rect_new_ex(upper_left_corner, size);
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
    let text_size = measure_text_ex(d.get_font_default(), text, font_size, 1.0);
    d.draw_text_ex(
        d.get_font_default(),
        text,
        upper_left_corner + size / 2.0 - text_size/2.0,
        font_size,
        1.0,
        Color::BLACK,
    );

    d.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) && hovered
}

struct TitleScreen {
    play_pong_game: bool,
    should_quit: bool,
}

impl TitleScreen {
    fn new() -> Self {
        TitleScreen {
            play_pong_game: false,
            should_quit: false,
        }
    }
}

impl Scene for TitleScreen {
    fn process(&mut self, rl: &RaylibHandle) {}
    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::GRAY);

        let screen_size = Vector2::new(d.get_screen_width() as f32, d.get_screen_height() as f32);

        let num_buttons = 2;
        let button_size = Vector2::new(250.0, 60.0);
        let set_of_buttons_size = button_size + Vector2::new(0.0, button_size.y*((num_buttons - 1) as f32));
        let spacing = 10.0;
        let mut cur_place_pos = screen_size / 2.0 - set_of_buttons_size / 2.0;
        
        if button(d, cur_place_pos, button_size, "PLAY") {
            self.play_pong_game = true;
        }
        cur_place_pos.y += button_size.y + spacing;

        if button(d, cur_place_pos, button_size, "EXIT") {
            self.should_quit = true;
        }

    }
    fn get_new_scene(&self) -> Option<Box<dyn Scene>> {
        if self.play_pong_game {
            Some(Box::new(pong::PongGame::new()))
        } else {
            None
        }
    }
    fn should_quit(&self) -> bool {
        self.should_quit
    }
}

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

    let mut cur_scene: Box<dyn Scene> = Box::new(TitleScreen::new());

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
