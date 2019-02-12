// engine.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

// #rustc This module is not based on the original C code.  It takes advantage
// of Rust's concurrency features to allow the engine to think on the opponent's
// time, while the main thread is awaiting input.

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::data::Data;

/// A command that can be sent to an Engine's command loop via its channel.
#[derive(Debug, Clone, Copy)]
enum Command {
    Stop,
}

/// An `Engine` is able to `think()` on a background thread, allowing the main
/// thread to handle I/O operations and higher-level game logic.
pub struct Engine {
    data: Arc<Mutex<Data>>,
    command_sender: Mutex<Option<Sender<Command>>>,
}

impl Engine {
    /// Create a new Engine instance.
    ///
    /// `start()` must be called before sending commands to the engine.
    ///
    /// # Example
    /// ```
    /// use tscp::engine::Engine;
    /// let engine = Engine::new();
    /// ```
    pub fn new() -> Engine {
        // TODO: Should new() also start()?  In that case, then drop() would
        // call stop(), which seems like a sensible way to manage the background
        // thread's lifetime.
        return Engine {
            data: Arc::new(Mutex::new(Data::new())),
            command_sender: Mutex::new(None),
        };
    }

    /// Start the engine's command-loop thread and establishes a channel.
    pub fn start(&mut self) {
        let mut cs = self.command_sender.lock().unwrap();
        match *cs {
            Some(_) => {
                panic!("attempt to start already-started engine");
            }
            None => {
                let (sender, receiver) = channel();
                thread::spawn(move || {
                    loop {
                        match receiver.recv().unwrap() {
                            Command::Stop => {
                                // #rustc TODO: stop the think thread if it is running
                                return 0;
                            }
                        }
                    }
                });
                *cs = Some(sender);
            }
        }
    }

    /// Stop the engine's command-loop thread.
    ///
    /// # Example
    /// ```
    /// use tscp::engine::Engine;
    /// let mut e = Engine::new();
    /// let sender = e.start();
    /// // ...
    /// e.stop()
    pub fn stop(self) {
        let cs = self.command_sender.lock().unwrap();
        match &*cs {
            None => {
                panic!("attempt to stop already stopped engine");
            }
            Some(sender) => {
                sender.send(Command::Stop).unwrap();
            }
        }
    }
}
