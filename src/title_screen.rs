use raylib::prelude::*;

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::imui::*;
use crate::scene::*;
use crate::awaiting_opponent;
use crate::pong;

pub struct TitleScreen {
    should_quit: bool,
    failed_to_connect_to_lobby: bool,
}

impl TitleScreen {
    pub fn new() -> Self {
        TitleScreen {
            should_quit: false,
            failed_to_connect_to_lobby: false,
        }
    }
}

impl Scene for TitleScreen {
    fn process(&mut self, _s: &mut SceneAPI, rl: &mut RaylibHandle) {}
    fn draw(&mut self, _s: &mut SceneAPI, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::GRAY);

        let screen_size = Vector2::new(d.get_screen_width() as f32, d.get_screen_height() as f32);

        if self.failed_to_connect_to_lobby {
            let err = "Failed to connect to lobby server";
            d.draw_text(
                err,
                (screen_size.x / 2.0
                    - measure_text_ex(d.get_font_default(), err, 30.0, 1.0).x / 2.0)
                    as i32,
                30,
                30,
                Color::WHITE,
            );
        }

        let num_buttons = 3;
        let button_size = Vector2::new(700.0, 60.0);
        let set_of_buttons_size =
            button_size + Vector2::new(0.0, button_size.y * ((num_buttons - 1) as f32));
        let spacing = 10.0;
        let mut cur_place_pos = screen_size / 2.0 - set_of_buttons_size / 2.0;

        if button(d, cur_place_pos, button_size, "HOST") {
            match TcpStream::connect("localhost:3333") {
                Ok(mut stream) => {
                    println!("Successfully connected to server in port 3333");

                    let msg: [u8; 5] = [1, 0, 0, 0, 0];

                    stream.write(&msg).unwrap();
                    println!("Sent create lobby command, awaiting lobby code...");

                    let mut data = [0 as u8; 4]; // using 4 byte buffer
                    match stream.read_exact(&mut data) {
                        Ok(_) => {
                            let response: i32 = i32::from_le_bytes(data);

                            if response != 0 {
                                println!("New lobby created! Lobby code: {}", response);
                                _s.new_scene = Some(Box::new(
                                    awaiting_opponent::AwaitingOpponent::new(stream, response),
                                ));
                            } else {
                                println!("Error creating lobby");
                            }
                        }
                        Err(e) => {
                            println!("Failed to receive data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect: {}", e);
                    self.failed_to_connect_to_lobby = true;
                }
            }
        }
        cur_place_pos.y += button_size.y + spacing;

        if button(d, cur_place_pos, button_size, "JOIN FROM CLIPBOARD") {
            let lobby_code_string = d.get_clipboard_text().unwrap(); // TODO handle error where clipboard content is not a string, a utf8 error instead
            let lobby_code = lobby_code_string.parse::<i32>().unwrap(); // TODO handle error where clipboard content is not a proper lobby code

            match TcpStream::connect("localhost:3333") {
                Ok(mut stream) => {
                    println!("Successfully connected to server in port 3333");
                    println!("Requesting to join lobby {}", lobby_code);
                    let lobby_code_bytes = lobby_code.to_le_bytes();
                    let msg: [u8; 5] = [
                        2,
                        lobby_code_bytes[0],
                        lobby_code_bytes[1],
                        lobby_code_bytes[2],
                        lobby_code_bytes[3],
                    ];

                    stream.write(&msg).unwrap();
                    println!("Sent join lobby command, awaiting lobby code...");

                    // TODO refactor this into a function based off of the host code
                    let mut data = [0 as u8; 4]; // using 4 byte buffer
                    match stream.read_exact(&mut data) {
                        Ok(_) => {
                            let response: i32 = i32::from_le_bytes(data);

                            if response == 200 {
                                println!("Joined Lobby!");
                                _s.new_scene = Some(Box::new(pong::PongGame::new(stream, false)));
                            } else {
                                println!(
                                    "Error creating lobby, response from server: {}",
                                    response
                                );
                            }
                        }
                        Err(e) => {
                            println!("Failed to receive data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect: {}", e);
                    self.failed_to_connect_to_lobby = true;
                }
            }
        }
        cur_place_pos.y += button_size.y + spacing;

        if button(d, cur_place_pos, button_size, "EXIT") {
            self.should_quit = true;
        }
    }
    fn should_quit(&self) -> bool {
        self.should_quit
    }
}
