use std::io;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crossterm::event::{self as cevent, Event as CEvent, KeyEvent, KeyCode};

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<CEvent>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: KeyCode,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: KeyCode::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                loop {
                    match cevent::poll(config.tick_rate) {
                        Ok(true) => {
                            match cevent::read() {
                                Ok(evt) => {
                                    if let Err(_) = tx.send(Event::Input(evt)) {
                                        return;
                                    }
                                    if !ignore_exit_key.load(Ordering::Relaxed)  {
                                        if let CEvent::Key(key_event) = evt {
                                            if key_event.code == config.exit_key {
                                                return;
                                            }
                                        }
                                    }
                                }
                                Err(err) => println!("{}", err),
                            }
                        },
                        Ok(false) => {}
                        Err(err) => println!("{}", err),
                    }

//                for evt in stdin.keys() {
//                    match evt {
//                        Ok(key) => {
//                            if let Err(_) = tx.send(Event::Input(key)) {
//                                return;
//                            }
//                            if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
//                                return;
//                            }
//                        }
//                        Err(_) => {}
//                    }
//                }
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<CEvent>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}
