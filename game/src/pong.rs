use crate::scene::*;

use raylib::prelude::*;

use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::vec::Vec;

use common::PongInputState;

pub const GAME_CONFIG: PongGameConfig = PongGameConfig {
    arena_size: Vector2::new(1000.0, 800.0),
    paddle_size: Vector2::new(25.0, 175.0),
    paddle_force: 1000.0,
    paddle_friction: 300.0,
    ball_size: 20.0,
    ball_speed: 400.0,
    score_font_size: 80,
    max_rollback_frames: 128,
    dt: 1.0 / 60.0,
};

pub struct PongGameConfig {
    pub arena_size: Vector2,
    paddle_size: Vector2,
    paddle_force: f32,
    paddle_friction: f32,
    ball_size: f32,
    ball_speed: f32,
    score_font_size: i32,
    max_rollback_frames: usize, // must be a power of 2 for RingBuffer
    dt: f32,
}

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

#[derive(PartialEq, Debug, Clone)]
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
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_v(self.position, GAME_CONFIG.paddle_size, Color::BLACK);
    }
}

#[derive(PartialEq, Debug, Clone)]
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

    fn draw(&self, d: &mut RaylibDrawHandle) {
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

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
struct PongGameState {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    left_score: Score,
    right_score: Score,
}

impl PongGameState {
    fn new() -> PongGameState {
        PongGameState {
            left_paddle: Paddle::new(true),
            right_paddle: Paddle::new(false),
            ball: Ball::new(1.0),
            left_score: Score::new(true),
            right_score: Score::new(false),
        }
    }

    fn process_paddle_input(&mut self, i: f32, is_on_left_side: bool) {
        (if is_on_left_side {
            &mut self.left_paddle
        } else {
            &mut self.right_paddle
        })
        .process_movement(i, GAME_CONFIG.dt);
    }

    fn process_logic(&mut self, inputs: &[PongInputState; 2]) {
        self.process_paddle_input(inputs[0].input, true);
        self.process_paddle_input(inputs[1].input, false);

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

    fn draw(&self, d: &mut RaylibDrawHandle) {
        self.left_paddle.draw(d);
        self.right_paddle.draw(d);
        self.ball.draw(d);
        self.left_score.draw(d);
        self.right_score.draw(d);
    }
}

struct PongInputAndGameState {
    player_inputs: [PongInputState; 2], // 0 is left, 1 is right
    game_after_inputs: PongGameState,
}

pub struct PongGame {
    cur_frame: u32,
    // TODO choose a different datastructure for this that does not have O(n) insert time...
    last_frames: Vec<PongInputAndGameState>, // This vector should always be guaranteed to have something in it, the initial state of the game
    future_inputs: Vec<PongInputState>,
    opponent_stream: TcpStream,
    playing_on_left_side: bool,

    // debug info
    frames_rolled_back: u32,
    oldest_frame_delay: u32,
}

impl PongGame {
    // is_host: the host is the left paddle, joiner is the right
    pub fn new(opponent_stream: TcpStream, is_host: bool) -> PongGame {
        opponent_stream.set_nonblocking(true).unwrap();
        PongGame {
            cur_frame: 0,
            last_frames: vec![PongInputAndGameState {
                player_inputs: [PongInputState::new(), PongInputState::new()],
                game_after_inputs: PongGameState::new(),
            }],
            future_inputs: Vec::new(),
            playing_on_left_side: is_host,
            opponent_stream: opponent_stream,
            frames_rolled_back: 0,
            oldest_frame_delay: 0,
        }
    }
}

impl Scene for PongGame {
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::WHITE);
        self.last_frames[0].game_after_inputs.draw(d);

        // debug drawing
        d.draw_text(
            &format!("FRMS ROLLED BACK: {}", self.frames_rolled_back),
            0,
            0,
            12,
            Color::RED,
        );
        d.draw_text(
            &format!(
                "OLDEST INPUT LATENCY: {} ms",
                ((self.oldest_frame_delay as f32) * GAME_CONFIG.dt) * 1000.0
            ),
            0,
            30,
            12,
            Color::RED,
        );
    }

    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle) {
        // construct local input from keys pressed
        let local_input = PongInputState {
            frame: self.cur_frame,
            input: dimension_strength(&rl, KeyboardKey::KEY_S, KeyboardKey::KEY_W),
        };
        self.opponent_stream.write(&local_input.into_u8()).unwrap();

        // TODO continually check the opponent stream until there are no more input state
        // chunks to process

        // fetch all input states available
        let mut remote_input_data = PongInputState::new().into_u8();
        let mut remote_inputs: Vec<PongInputState> = Vec::new();
        loop {
            match self.opponent_stream.read_exact(&mut remote_input_data) {
                Ok(_) => {
                    // println!("Received input state, processing input...");
                    remote_inputs.push(unsafe { PongInputState::from_u8(remote_input_data) });
                }
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => {}
                        _ => {
                            println!("Failed to receive data from server: {}", e);
                        }
                    }
                    break;
                }
            }
        }

        let mut cur_frame_inputs: [Option<PongInputState>; 2] = [None, None];

        let remote_player_index: usize;
        let local_player_index: usize;
        if self.playing_on_left_side {
            remote_player_index = 1;
            local_player_index = 0;
        } else {
            remote_player_index = 0;
            local_player_index = 1;
        }

        self.frames_rolled_back = 0; // updated to not zero if need to roll back
        self.oldest_frame_delay = 0;
        if remote_inputs.len() == 0 {
            // no remote input received, copy the last input for the appropriate remote player.
        } else {
            // rollback and input duplication logic
            for remote_input in remote_inputs {
                if remote_input.frame > self.cur_frame {
                    // input arrived from the future?? I think this means that I'm running behind, so
                    // TODO use this offset to figure out how much of the current frame to skip maybe
                    self.future_inputs.push(remote_input);
                    rl.set_target_fps(61);
                    continue;
                }
                let frame_offset = self.cur_frame - remote_input.frame;
                if frame_offset == 0 {
                    // remote input just happened
                    cur_frame_inputs[remote_player_index] = Some(remote_input);
                    rl.set_target_fps(60);
                } else if frame_offset > 0 {
                    // remote input happened frame_offset frames in the past

                    // rollback to the gamestate that many frames ago
                    let mut cur_game_state_index: i32 = (frame_offset - 1) as i32; // it's not zero, and zero was the last frame's game state

                    // assert that the frame did not happen too far in the past
                    // TODO do something like pause the game until everything can be resynced in case this happens
                    assert!((cur_game_state_index as usize) < self.last_frames.len());

                    if frame_offset > 1 {
                        // TODO see how many frames of latency should be expected with occasional ping test
                        // or something then only slow down if there's more frames of latency than expected from the network
                        rl.set_target_fps(59);
                    }

                    self.oldest_frame_delay = self.oldest_frame_delay.max(frame_offset);

                    // set the input of that frame to what was received
                    let must_rollback = self.last_frames[cur_game_state_index as usize]
                        .player_inputs[remote_player_index]
                        .input
                        != remote_input.input;

                    if must_rollback {
                        self.last_frames[cur_game_state_index as usize].player_inputs
                            [remote_player_index] = remote_input;
                        self.frames_rolled_back = frame_offset;
                    }

                    // in the while loop, after I update the frame's inputs that the remote told me,
                    // all the future frame's inputs are assumed to be duplicating the wrong frame.
                    // TODO for UDP packets I think that because stuff can arrive out of order,
                    // I need to be marking the frames that are duplicates of the previous one
                    let mut copy_inputs = false;

                    // resimulate all the frames that I rolled back, if the input was different
                    while must_rollback && cur_game_state_index >= 0 {
                        // rewind the game state to the previous frame's game state
                        let previous_game_state: PongGameState;
                        assert!(((cur_game_state_index + 1) as usize) < self.last_frames.len());
                        previous_game_state = self.last_frames[(cur_game_state_index + 1) as usize]
                            .game_after_inputs
                            .clone();

                        self.last_frames[cur_game_state_index as usize].game_after_inputs =
                            previous_game_state;

                        if copy_inputs {
                            self.last_frames[cur_game_state_index as usize].player_inputs =
                                self.last_frames[(cur_game_state_index + 1) as usize].player_inputs;
                        }

                        copy_inputs = true;

                        // resimulate the current frame with its updated inputs
                        let cur_frame: &mut PongInputAndGameState =
                            &mut self.last_frames[cur_game_state_index as usize];
                        cur_frame
                            .game_after_inputs
                            .process_logic(&cur_frame.player_inputs);

                        cur_game_state_index -= 1;
                    }
                }
            }
        }

        if cur_frame_inputs[remote_player_index].is_none() {
            // check the future frame cache
            let mut successfully_used_future_frame_cache = false;
            if self.future_inputs.len() > 0 {
                assert!(self.future_inputs[0].frame >= self.cur_frame); // assert that it's in the future or right now
                if self.future_inputs[0].frame == self.cur_frame {
                    cur_frame_inputs[remote_player_index] = Some(self.future_inputs.remove(0));
                    successfully_used_future_frame_cache = true;
                }
            }

            // duplicate the last frame
            if !successfully_used_future_frame_cache {
                cur_frame_inputs[remote_player_index] =
                    Some(self.last_frames[0].player_inputs[remote_player_index]);
            }
        }

        let mut new_game_state = self.last_frames[0].game_after_inputs.clone();

        cur_frame_inputs[local_player_index] = Some(local_input);

        let cur_frame_inputs = [cur_frame_inputs[0].unwrap(), cur_frame_inputs[1].unwrap()];

        new_game_state.process_logic(&cur_frame_inputs);

        // println!("Sending my input...");

        self.last_frames.insert(
            0,
            PongInputAndGameState {
                player_inputs: cur_frame_inputs,
                game_after_inputs: new_game_state,
            },
        );

        if self.last_frames.len() > GAME_CONFIG.max_rollback_frames {
            self.last_frames.pop();
        }

        self.cur_frame += 1;
    }
    fn should_quit(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut states: [PongGameState; 7] = [
            PongGameState::new(),
            PongGameState::new(),
            PongGameState::new(),
            PongGameState::new(),
            PongGameState::new(),
            PongGameState::new(),
            PongGameState::new(),
        ];

        for s in states.iter_mut() {
            for frame in 0..5000 {
                let elapsed_time = (frame as f32) * GAME_CONFIG.dt;
                s.process_logic(&[
                    PongInputState::from_input(elapsed_time.sin()),
                    PongInputState::from_input(elapsed_time.cos()),
                ]);
            }
        }

        for s in states.iter().skip(1) {
            assert_eq!(&states[0], s);
        }
    }
}
