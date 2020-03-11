use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use log::debug;
use crossterm::event::{self as cevent, Event as CEvent, KeyCode};
use tokio::prelude::*;
use tokio::task;
use tokio::sync::mpsc::{self, Receiver, Sender, error::RecvError, error::TryRecvError};
use crate::mudnet::CnxOutput;
use futures::Future;

pub enum Event<I,N> {
    Input(I),
    Network(N),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: Receiver<Event<CEvent, CnxOutput>>,
    input_handle: task::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
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
            tick_rate: Duration::from_millis(100),
        }
    }
}

impl Events {
    pub fn new(mut network: Receiver<CnxOutput>) -> Events {
        Events::with_config(Config::default(), network)
    }

    pub fn with_config(config: Config,
                       mut network: Receiver<CnxOutput>) -> Events {
        let (mut tx, mut rx) = mpsc::channel(100);
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let ignore_exit_key = ignore_exit_key.clone();
            tokio::spawn(async move {
                loop {
                    match network.try_recv() {
                        Ok(msg) => {
                            debug!("receive cnx: {:?}", msg);
                            if let Err(_) = tx.send(Event::Network(msg)).await {
                                return;
                            }
                        }
                        Err(TryRecvError::Empty) => {
                            //debug!("try receive cnx empty !");
                            ()
                        }
                        Err(TryRecvError::Closed) => break,
                    };


                    match cevent::poll(config.tick_rate) {
                        Ok(true) => match cevent::read() {
                            Ok(evt) => {
                                if let Err(_) = tx.send(Event::Input(evt)).await {
                                    return;
                                }
                                if !ignore_exit_key.load(Ordering::Relaxed) {
                                    if let CEvent::Key(key_event) = evt {
                                        if key_event.code == config.exit_key {
                                            return;
                                        }
                                    }
                                }
                            }
                            Err(err) => println!("{}", err),
                        },
                        Ok(false) => {}
                        Err(err) => println!("{}", err),
                    }
                }
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
        }
    }

    pub async fn next(&mut self) -> Option<Event<CEvent,CnxOutput>> {
        self.rx.recv().await
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}
