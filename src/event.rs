use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::get_tty;
use termion::input::TermRead;

pub enum Event {
    Input(String),
    Key(Key),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event>,
    rx_keys: mpsc::Receiver<Event>,
    input_handle: thread::JoinHandle<()>,
    keyboard_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(60),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let (tx_keys, rx_keys) = mpsc::channel();

        // Reads stdin
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                let stdin = std::io::stdin();

                loop {
                    let mut input = String::new();
                    let read_bytes = stdin.read_line(&mut input).expect("Can't read stdin");
                    // End of file
                    if read_bytes == 0 {
                        break;
                    }

                    // If can't send close the channel
                    if let Err(_) = tx.send(Event::Input(input)) {
                        break;
                    }
                }
            })
        };

        // Reads tty
        let keyboard_handle = {
            let tx = tx_keys.clone();
            thread::spawn(move || {
                let tty = get_tty().unwrap();
                for evt in tty.keys() {
                    match evt {
                        Ok(key) => {
                            if let Err(_) = tx.send(Event::Key(key)) {
                                return;
                            }
                            if key == config.exit_key {
                                return;
                            }
                        }
                        Err(_) => {}
                    }
                }
            })
        };

        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    if let Err(_) = tx.send(Event::Tick) {
                        return;
                    }
                    thread::sleep(config.tick_rate);
                }
            })
        };

        Events {
            rx,
            rx_keys,
            input_handle,
            keyboard_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        // Try to receive first the keystrokes
        match self.rx_keys.try_recv() {
            Ok(value) => Ok(value),
            Err(_) => self.rx.recv(),
        }
    }
}
