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
        };
    }
    fn process_movement(&mut self, vertical_input: f32, dt: f32, screen_size: Vector2) {
        self.position.y += vertical_input * dt * self.size.y * 1.5;
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
    size: f32,
}

impl Ball {
    fn new(position: Vector2, movement: Vector2, size: f32) -> Ball {
        return Ball {
            position: position,
            movement: movement,
            size: size,
        };
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
        self.position += self.movement * dt * speed;
        if left_paddle.ball_overlaps(self) || right_paddle.ball_overlaps(self) {
            self.movement.x *= -1.0;
        }
        if self.position.y <= self.size || self.position.y >= screen_size.y - self.size {
            self.movement.y *= -1.0;
        }
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
    let ball_speed = paddle_size.x * 10.0;

    let mut l_paddle = Paddle::new(paddle_size, screen_size, true);
    let mut r_paddle = Paddle::new(paddle_size, screen_size, false);
    let mut ball = Ball::new(
        screen_size / 2.0,
        Vector2::new(1.0, -1.0).normalized(),
        ball_size,
    );

    while !rl.window_should_close() {
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
        // check for ball collisions

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        l_paddle.draw(&mut d);
        r_paddle.draw(&mut d);
        ball.draw(&mut d);
    }
}
