use raylib::prelude::*;

fn clamp(d: f32, min: f32, max: f32) -> f32 {
    let t;
    if d < min {
        t = min;
    } else {
        t = d;
    }
    if t > max {
        return max;
    } else {
        return t;
    }
}

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
    size: Vector2,
    velocity: f32,
}

impl Paddle {
    fn new(size: Vector2, screen_size: Vector2, on_left_side: bool) -> Paddle {
        let pos: Vector2;
        if on_left_side {
            pos = Vector2::new(0.0, 0.0);
        } else {
            pos = Vector2::new(screen_size.x - size.x, 0.0);
        }
        return Paddle {
            position: pos,
            size: size,
            velocity: 0.0,
        };
    }
    fn process_movement(&mut self, vertical_input: f32, dt: f32, screen_size: Vector2) {
        self.velocity = vertical_input * self.size.y * 1.5;
        self.position.y += self.velocity * dt;
        self.position.y = clamp(self.position.y, 0.0, screen_size.y - self.size.y);
    }
    fn ball_overlaps(&self, ball: &Ball) -> bool {
        let local_ball_pos = ball.position - self.position;
        return (local_ball_pos.x >= -ball.size && local_ball_pos.x <= self.size.x + ball.size)
            && (local_ball_pos.y >= -ball.size && local_ball_pos.y <= self.size.y + ball.size);
    }
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_v(self.position, self.size, Color::BLACK);
    }
}

struct Ball {
    position: Vector2,
    movement: Vector2,
    increased_speed: f32,
    size: f32,
}

impl Ball {
    fn new(size: f32) -> Ball {
        return Ball {
            position: Vector2::new(0.0, 0.0),
            movement: Vector2::new(1.0, 0.0),
            increased_speed: 0.0,
            size: size,
        };
    }

    fn reset(&mut self, screen_size: Vector2) {
        self.position = screen_size / 2.0;
        self.movement = Vector2::new(self.movement.x*-1.0, 0.0).normalized();
        self.increased_speed = 0.0;
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.position, self.size, Color::RED);
    }
    // Moves along the movement vector and bounces on paddles
    fn process_movement(
        &mut self,
        dt: f32,
        speed: f32,
        screen_size: Vector2,
        left_paddle: &Paddle,
        right_paddle: &Paddle,
    ) {
        if left_paddle.ball_overlaps(self) {
            self.movement.x *= -2.0;
            self.position.x = left_paddle.position.x + left_paddle.size.x + self.size;
            self.movement.y += sign(left_paddle.velocity);
            self.movement.normalize();
        }
        if right_paddle.ball_overlaps(self) {
            self.movement.x *= -2.0;
            self.position.x = right_paddle.position.x - self.size;
            self.movement.y += sign(right_paddle.velocity);
            self.movement.normalize();
        }
        if self.position.y <= self.size {
            self.movement.y *= -1.0;
            self.position.y = self.size;
        }
        if self.position.y >= screen_size.y - self.size {
            self.movement.y *= -1.0;
            self.position.y = screen_size.y - self.size;
        }
        self.position += self.movement * dt * (speed + self.increased_speed);
        self.increased_speed += dt * 30.0;
    }
}

struct Score {
    value: i32,
    size: i32,
    left_side: bool,
}

impl Score {
    fn draw(&self, d: &mut RaylibDrawHandle, screen_size: Vector2) {
        let score_string = self.value.to_string();
        let to_draw_middle_x;
        if self.left_side {
            to_draw_middle_x = screen_size.x / 4.0;
        } else {
            to_draw_middle_x = (3.0 * screen_size.x) / 4.0;
        }
        d.draw_text(
            &score_string,
            to_draw_middle_x as i32 - (measure_text(&score_string, self.size) / 2),
            20,
            self.size,
            Color::BLACK,
        );
    }
}

fn main() {
    let screen_size = Vector2::new(1000.0, 800.0);
    let (mut rl, thread) = raylib::init()
        .size(screen_size.x as i32, screen_size.y as i32)
        .title("Rust Pong")
        .build();

    let paddle_size = Vector2::new(25.0, 175.0);
    let ball_size = 20.0;
    let mut ball_speed = paddle_size.x * 20.0;
    let starting_ball_position = screen_size / 2.0;
    let score_font_size = 80;

    let mut l_paddle = Paddle::new(paddle_size, screen_size, true);
    let mut r_paddle = Paddle::new(paddle_size, screen_size, false);
    let mut ball = Ball::new(ball_size);
    ball.reset(screen_size);

    let mut l_score = Score {
        value: 0,
        size: score_font_size,
        left_side: true,
    };
    let mut r_score = Score {
        value: 0,
        size: score_font_size,
        left_side: false,
    };

    while !rl.window_should_close() {
        ball_speed += rl.get_frame_time() * 10.0;

        l_paddle.process_movement(
            dimension_strength(&rl, KeyboardKey::KEY_S, KeyboardKey::KEY_W),
            rl.get_frame_time(),
            screen_size,
        );
        r_paddle.process_movement(
            dimension_strength(&rl, KeyboardKey::KEY_K, KeyboardKey::KEY_I),
            rl.get_frame_time(),
            screen_size,
        );

        ball.process_movement(
            rl.get_frame_time(),
            ball_speed,
            screen_size,
            &l_paddle,
            &r_paddle,
        );

        if ball.position.x <= -ball_size {
            r_score.value += 1;
            ball.reset(screen_size);
        }
        if ball.position.x >= screen_size.x + ball_size {
            l_score.value += 1;
            ball.reset(screen_size);
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        l_paddle.draw(&mut d);
        r_paddle.draw(&mut d);
        ball.draw(&mut d);
        l_score.draw(&mut d, screen_size);
        r_score.draw(&mut d, screen_size);
    }
}
