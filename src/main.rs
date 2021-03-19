use raylib::prelude::*;

const SCREEN_SIZE: Vector2 = Vector2::new(1000.0, 800.0);
const GAME_CONFIG: PongGameConfig = PongGameConfig {
    paddle_size: Vector2::new(25.0, 175.0),
    paddle_force: 1000.0,
    paddle_friction: 300.0,
    ball_size: 20.0,
    ball_speed: 400.0,
    score_font_size: 80,
};

fn key_strength(rl: &RaylibHandle, key: KeyboardKey) -> f32 {
    if rl.is_key_down(key) {
        1.0
    } else {
        0.0
    }
}

fn dimension_strength(
    rl: &RaylibHandle,
    positive_key: KeyboardKey,
    negative_key: KeyboardKey,
) -> f32 {
    key_strength(rl, positive_key) - key_strength(rl, negative_key)
}

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

struct Paddle {
    position: Vector2,
    velocity: f32,
    on_left_side: bool,
}

impl Paddle {
    fn new(on_left_side: bool) -> Paddle {
        let pos = if on_left_side {
            Vector2::new(0.0, 0.0)
        } else {
            Vector2::new(SCREEN_SIZE.x - GAME_CONFIG.paddle_size.x, 0.0)
        };

        Paddle {
            position: pos,
            velocity: 0.0,
            on_left_side: on_left_side,
        }
    }
    fn process_movement(&mut self, vertical_input: f32, dt: f32) {
        self.velocity +=
            vertical_input * (GAME_CONFIG.paddle_force + GAME_CONFIG.paddle_friction) * dt;
        let friction_effect = -self.velocity.signum() * GAME_CONFIG.paddle_friction * dt;
        if self.velocity.abs() < friction_effect.abs() {
            self.velocity = 0.0;
        } else {
            self.velocity += friction_effect;
        }
        self.position.y += self.velocity * dt;
        if self.position.y <= 0.0 || self.position.y + GAME_CONFIG.paddle_size.y >= SCREEN_SIZE.y {
            self.velocity *= -1.0;
        }
    }
    fn get_ball_hit_x(&self) -> f32 {
        if self.on_left_side {
            self.position.x + GAME_CONFIG.paddle_size.x + GAME_CONFIG.ball_size
        } else {
            self.position.x - GAME_CONFIG.ball_size
        }
    }
    fn ball_overlaps(&self, ball: &Ball) -> bool {
        let local_ball_pos = ball.position - self.position;

        (local_ball_pos.x >= -GAME_CONFIG.ball_size
            && local_ball_pos.x <= GAME_CONFIG.paddle_size.x + GAME_CONFIG.ball_size)
            && (local_ball_pos.y >= -GAME_CONFIG.ball_size
                && local_ball_pos.y <= GAME_CONFIG.paddle_size.y + GAME_CONFIG.ball_size)
    }
    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_v(self.position, GAME_CONFIG.paddle_size, Color::BLACK);
    }
}

struct Ball {
    position: Vector2,
    movement: Vector2,
    increased_speed: f32,
}

impl Ball {
    fn new() -> Ball {
        let mut to_return = Ball {
            position: Vector2::new(0.0, 0.0),
            movement: Vector2::new(1.0, 0.0),
            increased_speed: 0.0,
        };
        to_return.reset();

        to_return
    }

    fn reset(&mut self) {
        self.position = SCREEN_SIZE / 2.0;
        self.movement = Vector2::new(self.movement.x * -1.0, 0.0).normalized();
        self.increased_speed = 0.0;
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.position, GAME_CONFIG.ball_size, Color::RED);
    }

    /// Moves along the movement vector and bounces on paddles
    fn process_movement(&mut self, dt: f32, left_paddle: &Paddle, right_paddle: &Paddle) {
        // bounce off of paddles
        let paddles = [left_paddle, right_paddle];
        for paddle in paddles.iter() {
            if paddle.ball_overlaps(self) {
                self.movement.x *= -2.0;
                self.position.x = paddle.get_ball_hit_x();
                if paddle.velocity.abs() > 0.01 {
                    self.movement.y += paddle.velocity.signum();
                }
                self.movement.normalize();
            }
        }

        // bounce off of top and bottom walls
        if self.position.y <= GAME_CONFIG.ball_size {
            self.movement.y *= -1.0;
            self.position.y = GAME_CONFIG.ball_size;
        }
        if self.position.y >= SCREEN_SIZE.y - GAME_CONFIG.ball_size {
            self.movement.y *= -1.0;
            self.position.y = SCREEN_SIZE.y - GAME_CONFIG.ball_size;
        }

        // move and increase speed over time
        self.position += self.movement * dt * (GAME_CONFIG.ball_speed + self.increased_speed);
        self.increased_speed += dt * 50.0;
    }
}

struct Score {
    value: i32,
    left_side: bool,
}

impl Score {
    fn new(on_left_side: bool) -> Score {
        Score {
            value: 0,
            left_side: on_left_side,
        }
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let score_string = self.value.to_string();
        let to_draw_middle_x;
        if self.left_side {
            to_draw_middle_x = SCREEN_SIZE.x / 4.0;
        } else {
            to_draw_middle_x = (3.0 * SCREEN_SIZE.x) / 4.0;
        }
        d.draw_text(
            &score_string,
            to_draw_middle_x as i32
                - (measure_text(&score_string, GAME_CONFIG.score_font_size) / 2),
            20,
            GAME_CONFIG.score_font_size,
            Color::BLACK,
        );
    }
}

struct PongGameConfig {
    paddle_size: Vector2,
    paddle_force: f32,
    paddle_friction: f32,
    ball_size: f32,
    ball_speed: f32,
    score_font_size: i32,
}

struct PongGame {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    left_score: Score,
    right_score: Score,
}

impl PongGame {
    fn new() -> PongGame {
        PongGame {
            left_paddle: Paddle::new(true),
            right_paddle: Paddle::new(false),
            ball: Ball::new(),
            left_score: Score::new(true),
            right_score: Score::new(false),
        }
    }
}

impl Scene for PongGame {
    fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::WHITE);
        self.left_paddle.draw(d);
        self.right_paddle.draw(d);
        self.ball.draw(d);
        self.left_score.draw(d);
        self.right_score.draw(d);
    }
    fn process(&mut self, rl: &RaylibHandle) {
        let dt = rl.get_frame_time();
        self.left_paddle.process_movement(
            dimension_strength(&rl, KeyboardKey::KEY_S, KeyboardKey::KEY_W),
            dt,
        );
        self.right_paddle.process_movement(
            dimension_strength(&rl, KeyboardKey::KEY_K, KeyboardKey::KEY_I),
            dt,
        );
        self.ball
            .process_movement(dt, &self.left_paddle, &self.right_paddle);
        if self.ball.position.x <= -GAME_CONFIG.ball_size {
            self.right_score.value += 1;
            self.ball.reset();
        }
        if self.ball.position.x >= SCREEN_SIZE.x + GAME_CONFIG.ball_size {
            self.left_score.value += 1;
            self.ball.reset();
        }
    }
    fn get_new_scene(&self) -> Option<Box<dyn Scene>> {
        None
    }
}

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
            Some(Box::new(PongGame::new()))
        } else {
            None
        }
    }
}

trait Scene {
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
