#[cfg(target_os = "windows")]
use crossterm::event::{self, Event, KeyCode};
use log::*;
use simplelog::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
#[cfg(target_os = "linux")]
use std::{
    io::{self, Read, Write},
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration},
};
#[cfg(target_os = "linux")]
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};

pub mod paths;
pub mod sql;
pub mod strings;

pub fn setup_logger(level_filter: LevelFilter) {
	let logger_config = ConfigBuilder::new()
		.set_time_offset_to_local().expect("Failed to get local time offset")
		.set_time_format_custom(format_description!("[hour]:[minute]:[second].[subsecond digits:3]"))
		.build();
	CombinedLogger::init(
		vec![
			TermLogger::new(level_filter, logger_config, TerminalMode::Mixed, ColorChoice::Auto),
			// TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
			// WriteLogger::new(LevelFilter::Error, Config::default(), File::create("my_rust_binary.log").unwrap()),
		]
	).unwrap();
}

pub fn watch_for_quit(keep_going: Arc<AtomicBool>) {
    #[cfg(target_os = "windows")]
    {
        loop {
            // event::read() is blocking and waits for the next event
            match event::read() {
                Ok(Event::Key(key_event)) => {
                    if key_event.code == KeyCode::Char('q') {
                        println!("Quit key 'q' pressed.");
                        break; // Exit the input thread loop
                    }
                },
                Ok(_) => {
                    // Ignore other events (like mouse or resize)
                },
                Err(e) => {
                    eprintln!("\nInput thread error: {}. Shutting down.", e);
                    break;
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let stdin = 0;
        let termios = Termios::from_fd(stdin).unwrap();
        let mut new_termios = termios.clone();  // make a mutable copy of termios that we will modify
        new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
        tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

        let (tx, rx) = mpsc::channel::<u8>();
        // Spawn the key_press_watcher_linux thread, passing the sender (tx) into it.
        _ = thread::spawn(move || {key_press_watcher_linux(tx);});

        //poll Q
        let mut key_seq:Vec<u8> = Vec::new();
        loop {
            match rx.try_recv() {
                // Case 1: A byte was successfully received.
                Ok(byte) => {
                    //this picks up all bytes in the queue and stored them in key_seq at once, so there is no need to check for time between ESC and other codes
                    // println!("[Consumer] Read byte: {}", byte);
                    key_seq.push(byte);
                }
                // Case 2: The queue is currently empty (No message available).
                Err(mpsc::TryRecvError::Empty) => {
                    // no keypress byte to process
                    if !key_seq.is_empty(){
                        if key_seq[0] == 113 || key_seq[0] == 81 {
                            //Q or q key pressed
                            println!("Quit key 'q' pressed.");
                            break;
                        }
                        //println!("key_seq: {:?}", key_seq);
                        key_seq.clear();
                    }

                }
                // Case 3: The sender (producer thread) has hung up or panicked.
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("get_key_sequence(): Sender disconnected");
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(20));
        }

        tcsetattr(stdin, TCSANOW, & termios).unwrap();  // reset the stdin to original termios data
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        keep_going.store(false, Ordering::Relaxed);
        panic!("Key watching not coded for this OS");
    }

    keep_going.store(false, Ordering::Relaxed);
}

#[cfg(target_os = "linux")]
fn key_press_watcher_linux(tx: Sender<u8>) {
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0;1];  // read exactly one byte
    stdout.lock().flush().unwrap();
    loop {
        //reader.read_exact(&mut buffer).unwrap();
        match reader.read(&mut buffer) {
            Ok(_) => {
                //println!("len_of_buffer: {}", len_of_buffer);
            }
            Err(e) => {
                println!("key_press_watcher_linux reader err: {}", e);
            }
        }
        match tx.send(buffer[0]) {
            Ok(_) => {
                //println!("send success");
            }
            Err(_) => {
                //println!("send error: {:?}", e);
                return;
            }
        }
    }
}
