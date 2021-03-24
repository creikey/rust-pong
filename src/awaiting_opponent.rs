use raylib::prelude::*;

use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::imui::*;
use crate::scene::*;
use crate::pong;

pub struct AwaitingOpponent {
    pub lobby_stream: TcpStream,
    pub lobby_code: i32,
    text_to_copy_to_clipboard: Option<String>,
}

impl AwaitingOpponent {
    pub fn new(stream: TcpStream, lobby_code: i32) -> Self {
        stream.set_nonblocking(true).unwrap();
        AwaitingOpponent {
            lobby_stream: stream,
            lobby_code: lobby_code,
            text_to_copy_to_clipboard: None,
        }
    }
}

impl Scene for AwaitingOpponent {
    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle) {
        let mut receive_buffer = [0];
        match self.lobby_stream.read_exact(&mut receive_buffer) {
            Ok(_) => {
                if receive_buffer[0] == 1 {
                    // player joined our lobby!

                    _s.new_scene = Some(Box::new(pong::PongGame::new(
                        self.lobby_stream.try_clone().unwrap(),
                    )));
                } else {
                    println!("Wonky thing received from server: {}", receive_buffer[0]);
                }
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::WouldBlock => {}
                    _ => {
                        println!("Failed to receive data from server: {}", e);
                    }
                }
            }
        }
        match &self.text_to_copy_to_clipboard {
            Some(text) => {
                rl.set_clipboard_text(&text).unwrap(); // TODO definitely need to let the user know if I couldn't copy the lobby code to clipboard
                self.text_to_copy_to_clipboard = None;
            }
            None => (),
        };
    }
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::GRAY);

        let screen_size = Vector2::new(d.get_screen_width() as f32, d.get_screen_height() as f32);

        let num_sections = 2;
        let section_size = Vector2::new(900.0, 60.0);
        let entire_size =
            section_size + Vector2::new(0.0, section_size.y * ((1 - num_sections) as f32));
        let spacing = 10.0;
        let mut cur_place_pos = screen_size / 2.0 - entire_size / 2.0;
        if button(
            d,
            cur_place_pos,
            section_size,
            "COPY LOBBY CODE TO CLIPBOARD",
        ) {
            self.text_to_copy_to_clipboard = Some(self.lobby_code.to_string());
        }
        cur_place_pos.y += section_size.y + spacing;
        d.draw_text_ex(
            d.get_font_default(),
            "WAITING FOR PLAYER TO JOIN...",
            cur_place_pos,
            50.0,
            1.0,
            Color::BLACK,
        );
    }

    fn should_quit(&self) -> bool {
        false
    }
}
