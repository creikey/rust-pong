use raylib::prelude::*;

const SCREEN_SIZE: Vector2 = Vector2::new(1000.0, 800.0);
const GAME_CONFIG: GameConfig = GameConfig {
    paddle_size: Vector2::new(25.0, 175.0),
    paddle_force: 1000.0,
    paddle_friction: 300.0,
    ball_size: 20.0,
    ball_speed: 400.0,
    score_font_size: 80,
};

fn key_strength(rl: &RaylibHandle, key: KeyboardKey) -> f32 {
    if rl.is_key_down(key) {
        return 1.0;
    } else {
        return 0.0;
    }
}

fn sign(n: f32) -> f32 {
    if n.abs() <= 0.01 {
        return 0.0;
    }
    if n > 0.0 {
        return 1.0;
    } else {
        return -1.0;
    }
}

fn dimension_strength(
    rl: &RaylibHandle,
    positive_key: KeyboardKey,
    negative_key: KeyboardKey,
) -> f32 {
    return key_strength(rl, positive_key) - key_strength(rl, negative_key);
}

struct Paddle {
    position: Vector2,
    velocity: f32,
}

impl Paddle {
    fn new(on_left_side: bool) -> Paddle {
        let pos: Vector2;
        if on_left_side {
            pos = Vector2::new(0.0, 0.0);
        } else {
            pos = Vector2::new(SCREEN_SIZE.x - GAME_CONFIG.paddle_size.x, 0.0);
        }
        return Paddle {
            position: pos,
            velocity: 0.0,
        };
    }
    fn process_movement(&mut self, vertical_input: f32, dt: f32) {
        self.velocity +=
            vertical_input * (GAME_CONFIG.paddle_force + GAME_CONFIG.paddle_friction) * dt;
        let friction_effect = -sign(self.velocity) * GAME_CONFIG.paddle_friction * dt;
        if self.velocity.abs() < friction_effect {
            self.velocity = 0.0;
        } else {
            self.velocity += friction_effect;
        }
        self.position.y += self.velocity * dt;
        if self.position.y <= 0.0 || self.position.y + GAME_CONFIG.paddle_size.y >= SCREEN_SIZE.y {
            self.velocity *= -1.0;
        }
    }
    fn ball_overlaps(&self, ball: &Ball) -> bool {
        let local_ball_pos = ball.position - self.position;
        return (local_ball_pos.x >= -GAME_CONFIG.ball_size
            && local_ball_pos.x <= GAME_CONFIG.paddle_size.x + GAME_CONFIG.ball_size)
            && (local_ball_pos.y >= -GAME_CONFIG.ball_size
                && local_ball_pos.y <= GAME_CONFIG.paddle_size.y + GAME_CONFIG.ball_size);
    }
    fn draw(&self, d: &mut RaylibDrawHandle) {
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
        return to_return;
    }

    fn reset(&mut self) {
        self.position = SCREEN_SIZE / 2.0;
        self.movement = Vector2::new(self.movement.x * -1.0, 0.0).normalized();
        self.increased_speed = 0.0;
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.position, GAME_CONFIG.ball_size, Color::RED);
    }
    // Moves along the movement vector and bounces on paddles
    fn process_movement(&mut self, dt: f32, left_paddle: &Paddle, right_paddle: &Paddle) {
        if left_paddle.ball_overlaps(self) {
            self.movement.x *= -2.0;
            self.position.x =
                left_paddle.position.x + GAME_CONFIG.paddle_size.x + GAME_CONFIG.ball_size;
            self.movement.y += sign(left_paddle.velocity);
            self.movement.normalize();
        }
        if right_paddle.ball_overlaps(self) {
            self.movement.x *= -2.0;
            self.position.x = right_paddle.position.x - GAME_CONFIG.ball_size;
            self.movement.y += sign(right_paddle.velocity);
            self.movement.normalize();
        }
        if self.position.y <= GAME_CONFIG.ball_size {
            self.movement.y *= -1.0;
            self.position.y = GAME_CONFIG.ball_size;
        }
        if self.position.y >= SCREEN_SIZE.y - GAME_CONFIG.ball_size {
            self.movement.y *= -1.0;
            self.position.y = SCREEN_SIZE.y - GAME_CONFIG.ball_size;
        }
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

    fn draw(&self, d: &mut RaylibDrawHandle) {
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

struct GameConfig {
    paddle_size: Vector2,
    paddle_force: f32,
    paddle_friction: f32,
    ball_size: f32,
    ball_speed: f32,
    score_font_size: i32,
}

struct Game {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    left_score: Score,
    right_score: Score,
}

impl Game {
    fn new() -> Game {
        Game {
            left_paddle: Paddle::new(true),
            right_paddle: Paddle::new(false),
            ball: Ball::new(),
            left_score: Score::new(true),
            right_score: Score::new(false),
        }
    }

    fn process(&mut self, rl: &RaylibHandle, dt: f32) {
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

    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::WHITE);
        self.left_paddle.draw(d);
        self.right_paddle.draw(d);
        self.ball.draw(d);
        self.left_score.draw(d);
        self.right_score.draw(d);
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_SIZE.x as i32, SCREEN_SIZE.y as i32)
        .title("Rust Pong")
        .build();

    let mut game = Game::new();

    while !rl.window_should_close() {
        game.process(&rl, rl.get_frame_time());

        let mut d = rl.begin_drawing(&thread);

        game.draw(&mut d);
    }
}
