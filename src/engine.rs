// engine.rs
// Tom Kerrigan's Simple Chess Program (TSCP)
//
// Copyright 1997 Tom Kerrigan
//
// Rust port by Kristopher Johnson

// #rustc This module is not based on the original C code.  It takes advantage
// of Rust's concurrency features to allow the engine to think on the opponent's
// time, while the main thread is awaiting input.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::board::{gen, init_board, init_hash, makemove, takeback};
use crate::book::{close_book, open_book};
use crate::data::Data;
use crate::defs::{Int, Move, MoveBytes, DARK, LIGHT};
use crate::search::{think, ThinkOutput};
use crate::{parse_move, print_board, print_result};

/// A command that can be sent to an Engine's background thread via its channel.
#[derive(Debug, Clone)]
enum Command {
    CanTakeBack(u32, Sender<bool>),
    ClearPly,
    CloseBook,
    Gen,
    InitBoard,
    MakeMove(MoveBytes, Sender<bool>),
    OpenBook,
    ParseMove(String, Sender<Option<MoveBytes>>),
    PrintBoard,
    PrintResult,
    SetMaxTimeAndDepth(Int, Int),
    SetSide(Int),
    Side(Sender<Int>),
    Stop,
    TakeBack,
    Think(ThinkOutput, Sender<Move>),
}

/// An `Engine` is able to `think()` and perform other processing on a
/// background thread, allowing the main thread to handle I/O operations and
/// higher-level game logic.
pub struct Engine {
    command_tx: Option<Sender<Command>>,
    command_thread: Option<JoinHandle<()>>,
}

impl Engine {
    /// Create a new Engine instance.
    ///
    /// `start()` must be called before sending commands to the engine.
    ///
    /// # Example
    /// ```
    /// use tscp::engine::Engine;
    ///
    /// let engine = Engine::new();
    /// ```
    pub fn new() -> Engine {
        return Engine {
            command_tx: None,
            command_thread: None,
        };
    }

    /// Start the engine's command-loop thread.
    pub fn start(&mut self) {
        let (tx, rx) = channel();
        let handle = thread::spawn(move || {
            Engine::process_commands(rx);
        });
        self.command_tx = Some(tx);
        self.command_thread = Some(handle);
    }

    /// Stop the engine's command-loop thread.
    ///
    /// # Example
    /// ```
    /// use tscp::engine::Engine;
    ///
    /// let mut e = Engine::new();
    /// e.start();
    /// // ...
    /// e.stop();
    pub fn stop(&mut self) {
        let command_thread = self.command_thread.take();
        if let Some(thread) = command_thread {
            self.send_command(Command::Stop);
            thread.join().unwrap();
        }
    }

    // Call `board::init_board()` on the engine's data.
    pub fn init_board(&mut self) {
        self.send_command(Command::InitBoard);
    }

    /// Call `book::open_book()` on the engine's data.
    pub fn open_book(&mut self) {
        self.send_command(Command::OpenBook);
    }

    /// Call `book::close_book()` on the engine's data.
    pub fn close_book(&mut self) {
        self.send_command(Command::CloseBook);
    }

    /// Call `board::gen()` on the engine's data.
    pub fn gen(&mut self) {
        self.send_command(Command::Gen);
    }

    /// Set the `max_time` and `max_depth` parameters of the engine's data.
    pub fn set_max_time_and_depth(&mut self, max_time: Int, max_depth: Int) {
        self.send_command(Command::SetMaxTimeAndDepth(max_time, max_depth));
    }

    /// Call `search::think()` on the engine's data.
    ///
    /// # Return value
    ///
    /// Returns the computer's move.  The move may be an "empty" move (`value()
    /// == 0`), indicating there are no legal moves.
    pub fn think(&mut self, output: ThinkOutput) -> Move {
        let (tx, rx) = channel();
        self.send_command(Command::Think(output, tx));
        return rx.recv().unwrap();
    }

    /// Call `board::makemove()` on the engine's data.
    ///
    /// # Return value
    ///
    /// Returns `true` if the move was valid, or `false` if the move was not
    /// valid.  If `false` is returned, then no change was made to the engine's
    /// data.
    pub fn makemove(&mut self, m: MoveBytes) -> bool {
        let (tx, rx) = channel();
        self.send_command(Command::MakeMove(m, tx));
        return rx.recv().unwrap();
    }

    /// Reset `data.ply` to zero.
    pub fn clear_ply(&mut self) {
        self.send_command(Command::ClearPly);
    }

    /// Call `tscp::print_board()` on the engine's data.
    pub fn print_board(&self) {
        self.send_command(Command::PrintBoard);
    }

    /// Call `tscp::print_result()` on the engine's data.
    pub fn print_result(&self) {
        self.send_command(Command::PrintResult);
    }

    /// Determine whether `takeback()` can be called.
    ///
    /// # Return value
    ///
    /// Return `true` if it is valid to call `takeback()`.
    pub fn can_takeback(&self) -> bool {
        self.can_takeback_n(1)
    }

    /// Determine whether `takeback()` can be called N times.
    ///
    /// # Return value
    ///
    /// Return `true` if it is valid to call `takeback()` the specified number
    /// of times.
    pub fn can_takeback_n(&self, n: u32) -> bool {
        let (tx, rx) = channel();
        self.send_command(Command::CanTakeBack(n, tx));
        return rx.recv().unwrap();
    }

    /// Call `board::takeback()` on the engine's data.
    pub fn takeback(&mut self) {
        self.send_command(Command::TakeBack);
    }

    /// Call `tscp::parse_move()` on the engine's data.
    ///
    /// # Return value
    ///
    /// Returns `None` if the string is not a valid move command.  Otherwise,
    /// returns the specified Move.
    pub fn parse_move(&self, s: String) -> Option<MoveBytes> {
        let (tx, rx) = channel();
        self.send_command(Command::ParseMove(s, tx));
        return rx.recv().unwrap();
    }

    /// Determine which side is making a move.
    ///
    /// # Return value
    ///
    /// Returns the side to move, `LIGHT` or `DARK`.
    pub fn side(&self) -> Int {
        let (tx, rx) = channel();
        self.send_command(Command::Side(tx));
        return rx.recv().unwrap();
    }

    /// Set the side to move.
    pub fn set_side(&mut self, side: Int) {
        self.send_command(Command::SetSide(side));
    }

    /// Send a command to the background thread.
    fn send_command(&self, command: Command) {
        self.command_tx.as_ref().unwrap().send(command).unwrap();
    }

    /// Process commands until `Command::Stop` is received.
    ///
    /// This function runs in the background thread.
    fn process_commands(rx: Receiver<Command>) {
        let mut d = Data::new();
        init_hash(&mut d);

        loop {
            let command = rx.recv().unwrap();
            match command {
                Command::CanTakeBack(n, tx) => {
                    tx.send(d.hply >= (n as usize)).unwrap();
                }
                Command::ClearPly => {
                    d.ply = 0;
                }
                Command::CloseBook => {
                    close_book(&mut d);
                }
                Command::Gen => {
                    gen(&mut d);
                }
                Command::InitBoard => {
                    init_board(&mut d);
                }
                Command::OpenBook => {
                    open_book(&mut d);
                }
                Command::MakeMove(m, tx) => {
                    tx.send(makemove(&mut d, m)).unwrap();
                }
                Command::ParseMove(string, tx) => {
                    let m = parse_move(&d, &string);
                    if m == -1 {
                        tx.send(None).unwrap();
                    } else {
                        let m = d.gen_dat[m as usize].m.bytes();
                        tx.send(Some(m)).unwrap();
                    }
                }
                Command::PrintBoard => {
                    print_board(&d);
                }
                Command::PrintResult => {
                    print_result(&mut d);
                }
                Command::SetMaxTimeAndDepth(max_time, max_depth) => {
                    d.max_time = max_time;
                    d.max_depth = max_depth;
                }
                Command::SetSide(side) => match side {
                    LIGHT => {
                        d.side = LIGHT;
                        d.xside = DARK;
                    }
                    DARK => {
                        d.side = DARK;
                        d.xside = LIGHT;
                    }
                    _ => {
                        panic!("set_side: invalid argument {}", side);
                    }
                },
                Command::Side(tx) => {
                    tx.send(d.side).unwrap();
                }
                Command::Stop => {
                    return;
                }
                Command::TakeBack => {
                    takeback(&mut d);
                }
                Command::Think(output, tx) => {
                    think(&mut d, output);
                    tx.send(d.pv[0][0]).unwrap();
                }
            }
        }
    }
}

impl Drop for Engine {
    /// Ensure command-thread is stopped if running when `Engine` goes out of
    /// scope.
    fn drop(&mut self) {
        self.stop();
    }
}
