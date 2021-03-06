use rand;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use common::PongInputState;
use common::PORT;

fn funnel_packets(
    source_stream: &mut TcpStream,
    target_stream: &mut TcpStream,
) -> Result<(), io::Error> {
    let mut buffer = [0u8; size_of::<PongInputState>()];

    let mut to_return = Ok(());
    loop {
        match source_stream.read_exact(&mut buffer) {
            Ok(_) => {
                target_stream.write(&buffer).unwrap(); // TODO fix unclean error here where connection is forcibly closed
             
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => break, // no more packets to funnel this way
                _ => {
                    println!(
                        "While funnneling packets from hoster -> joiner, an error occured: {}",
                        e
                    );
                    source_stream.shutdown(Shutdown::Both).unwrap();
                    to_return = Err(e);
                    break;
                }
            },
        }
    }
    
    to_return
}

fn handle_client(
    mut stream: TcpStream,
    lobby_to_host_transmitter: Arc<Mutex<HashMap<i32, mpsc::Sender<TcpStream>>>>,
) {
    let mut data = [0 as u8; 5];
    let mut tx: Option<mpsc::Sender<TcpStream>> = None;
    let mut rx: Option<mpsc::Receiver<TcpStream>> = None;
    let mut my_lobby_code: Option<i32> = None;
    loop {
        match stream.read(&mut data) {
            Ok(size) => {
                if size != 5 {
                    println!("Got funky data size back (endl?), size: {}", size);
                    break;
                } else if data[0] == 1 {
                    // create lobby command
                    println!("Creating lobby...");
                    let (new_tx, new_rx) = mpsc::channel();
                    tx = Some(new_tx);
                    rx = Some(new_rx);
                    let mut dict = lobby_to_host_transmitter.lock().unwrap();
                    let tx_into_sender = tx.as_ref().unwrap().clone();
                    let new_lobby_id = rand::random::<i32>(); // TODO I don't check for lobby id conflicts here
                    (*dict).insert(new_lobby_id, tx_into_sender);
                    println!("Created lobby with ID: {}", new_lobby_id);
                    let encoded_lobby_id = new_lobby_id.to_le_bytes();
                    stream.write(&encoded_lobby_id).unwrap(); // TODO should probably cleanly handle failing to send lobby code
                    my_lobby_code = Some(new_lobby_id);
                    break;
                } else if data[0] == 2 {
                    // join lobby command

                    let received_lobby_code =
                        i32::from_le_bytes([data[1], data[2], data[3], data[4]]);
                    println!(
                        "Attempting to connect a client to lobby: {}",
                        received_lobby_code
                    );
                    let dict = lobby_to_host_transmitter.lock().unwrap();
                    if (*dict).contains_key(&received_lobby_code) {
                        println!(
                        "Lobby exists! Saying OK then Sending my stream to that lobby thread..."
                    );
                        stream.write(&200i32.to_le_bytes()).unwrap();
                        (*dict)
                            .get(&received_lobby_code)
                            .unwrap()
                            .send(stream)
                            .unwrap();

                        // returning from the entire function here so the borrow checker allows
                        // the sending of my stream to the dict
                        return;
                    } else {
                        println!("Lobby does not exist...");
                        stream.write(&400i32.to_le_bytes()).unwrap();
                    }
                } else {
                    println!("Weird data that's not a known command: {}", data[0]);
                    break;
                }
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
        }
    }

    if tx.is_some() && rx.is_some() {
        // If these are some, I am a thread that is managing a lobby and waiting for somebody to join
        // from the shared dict. Block on my channel until I receive a stream to connect with
        println!("Waiting for client to connect...");
        let mut other_stream = rx.unwrap().recv().unwrap(); // TODO need some way to timeout on this

        // let the inviter know that somebody has joined and they can start funneling packets now
        println!("Client connected! Letting the host know...");
        stream.write(&[1]).unwrap();

        // Now that I have both streams, I can funnel packets back and forth. First I remove
        // my lobby from the dict, as I am no longer waiting for a connection with recv
        let mut dict = lobby_to_host_transmitter.lock().unwrap();
        (*dict).remove_entry(&my_lobby_code.unwrap()).unwrap();

        // Don't block while polling between two sockets as reading/writing
        // from both with separate threads is not supported
        stream.set_nonblocking(true).unwrap();
        other_stream.set_nonblocking(true).unwrap();

        println!("Funneling packets between two clients in loop...");
        let mut e;
        loop {
            e = funnel_packets(&mut stream, &mut other_stream);
            if e.is_err() {
                break;
            }
            e = funnel_packets(&mut other_stream, &mut stream);
            if e.is_err() {
                break;
            }
            thread::sleep(time::Duration::from_millis(1));
        }

        println!("Stopped funneling packets: {}", e.err().unwrap());
    }
}

fn main() {
    // Hashmap is probably the wrong datastructure to use for this problem,
    // I don't care about the value of the keys, I just need each key to be
    // unique and be able to add/remove keys at will.
    let lobby_to_host_transmitter: Arc<Mutex<HashMap<i32, mpsc::Sender<TcpStream>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", PORT);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let reference_to_lobby_stuff = Arc::clone(&lobby_to_host_transmitter);
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream, reference_to_lobby_stuff);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}
