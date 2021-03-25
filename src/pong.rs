use crate::scene::*;

use raylib::prelude::*;

use std::io;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::TcpStream;

pub const GAME_CONFIG: PongGameConfig = PongGameConfig {
    arena_size: Vector2::new(1000.0, 800.0),
    paddle_size: Vector2::new(25.0, 175.0),
    paddle_force: 1000.0,
    paddle_friction: 300.0,
    ball_size: 20.0,
    ball_speed: 400.0,
    score_font_size: 80,
    dt: 1.0 / 60.0,
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
            Vector2::new(GAME_CONFIG.arena_size.x - GAME_CONFIG.paddle_size.x, 0.0)
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
        if self.position.y <= 0.0
            || self.position.y + GAME_CONFIG.paddle_size.y >= GAME_CONFIG.arena_size.y
        {
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
    fn new(horizontal_multiplier: f32) -> Ball {
        let mut to_return = Ball {
            position: Vector2::new(0.0, 0.0),
            movement: Vector2::new(1.0 * horizontal_multiplier, 0.0),
            increased_speed: 0.0,
        };
        to_return.reset();

        to_return
    }

    fn reset(&mut self) {
        self.position = GAME_CONFIG.arena_size / 2.0;
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
        if self.position.y >= GAME_CONFIG.arena_size.y - GAME_CONFIG.ball_size {
            self.movement.y *= -1.0;
            self.position.y = GAME_CONFIG.arena_size.y - GAME_CONFIG.ball_size;
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
            to_draw_middle_x = GAME_CONFIG.arena_size.x / 4.0;
        } else {
            to_draw_middle_x = (3.0 * GAME_CONFIG.arena_size.x) / 4.0;
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

#[repr(C)]
pub struct PongInputState {
    frame: i32,
    input: f32,
}

impl PongInputState {
    fn new() -> Self {
        PongInputState {
            frame: 0,
            input: 0.0,
        }
    }

    // To ensure byte alignment, you should probably
    // call this like PongInputState::new().into_u8()
    pub fn into_u8(self) -> [u8; size_of::<Self>()] {
        unsafe { std::mem::transmute(self) }
    }

    // caution: this method may only be called on u8 slices that are slices of memory that is PongInputState
    // or at least PongInputState byte aligned.
    pub unsafe fn from_u8(b: [u8; size_of::<Self>()]) -> Self {
        std::mem::transmute(b)
    }
}

pub struct PongGameConfig {
    pub arena_size: Vector2,
    paddle_size: Vector2,
    paddle_force: f32,
    paddle_friction: f32,
    ball_size: f32,
    ball_speed: f32,
    score_font_size: i32,
    dt: f32,
}

pub struct PongGame {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    left_score: Score,
    right_score: Score,
    opponent_stream: TcpStream,
    playing_on_left_side: bool,
}

impl PongGame {
    // is_host: the host serves the ball first
    pub fn new(opponent_stream: TcpStream, is_host: bool) -> PongGame {
        opponent_stream.set_nonblocking(true).unwrap();
        PongGame {
            left_paddle: Paddle::new(true),
            right_paddle: Paddle::new(false),
            ball: Ball::new(1.0),
            left_score: Score::new(true),
            right_score: Score::new(false),
            playing_on_left_side: is_host,
            opponent_stream: opponent_stream,
        }
    }

    fn process_input(&mut self, i: &PongInputState, is_on_left_side: bool) {
        (if is_on_left_side {
            &mut self.left_paddle
        } else {
            &mut self.right_paddle
        })
        .process_movement(i.input, GAME_CONFIG.dt);
    }
}

impl Scene for PongGame {
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::WHITE);
        self.left_paddle.draw(d);
        self.right_paddle.draw(d);
        self.ball.draw(d);
        self.left_score.draw(d);
        self.right_score.draw(d);
    }

    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle) {
        let cur_input_state = PongInputState {
            frame: 0,
            input: dimension_strength(&rl, KeyboardKey::KEY_S, KeyboardKey::KEY_W),
        };

        {
            let mut data = PongInputState::new().into_u8();
            match self.opponent_stream.read_exact(&mut data) {
                Ok(_) => {
                    // println!("Received input state, processing input...");
                    self.process_input(unsafe { &PongInputState::from_u8(data) }, !self.playing_on_left_side);
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => {
                        println!("Failed to receive data from server: {}", e);
                    }
                },
            }
        }
        self.process_input(&cur_input_state, self.playing_on_left_side);

        // println!("Sending my input...");
        self.opponent_stream
            .write(&cur_input_state.into_u8())
            .unwrap();

        let dt = GAME_CONFIG.dt;
        self.ball
            .process_movement(dt, &self.left_paddle, &self.right_paddle);
        if self.ball.position.x <= -GAME_CONFIG.ball_size {
            self.right_score.value += 1;
            self.ball.reset();
        }
        if self.ball.position.x >= GAME_CONFIG.arena_size.x + GAME_CONFIG.ball_size {
            self.left_score.value += 1;
            self.ball.reset();
        }
    }
    fn should_quit(&self) -> bool {
        false
    }
}
