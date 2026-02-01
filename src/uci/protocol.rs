//! UCI Protocol implementation
//!
//! This module handles the Universal Chess Interface protocol,
//! parsing commands from GUIs and sending responses.

use crate::bitboard::position::Position;
use crate::eval::Evaluator;
use crate::movegen::Move;
use crate::search::alphabeta::{iterative_deepening, SearchResult};
use crate::search::transposition::TranspositionTable;
use crate::uci::commands::{parse_command, TimeControl, UciCommand};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

/// UCI Engine state
pub struct UciEngine {
    position: Position,
    evaluator: Evaluator,
    tt: TranspositionTable,
    time_control: TimeControl,
    stop_flag: Arc<AtomicBool>,
    search_handle: Option<thread::JoinHandle<()>>,
    result_sender: mpsc::Sender<SearchResult>,
    result_receiver: mpsc::Receiver<SearchResult>,
}

impl UciEngine {
    /// Create a new UCI engine
    pub fn new() -> Self {
        // Initialize magic bitboards (must be done once at startup)
        crate::bitboard::magic::init_magics();

        let mut position = Position::empty();
        position.set_startpos();

        let (tx, rx) = mpsc::channel();

        UciEngine {
            position,
            evaluator: Evaluator::new(),
            tt: TranspositionTable::new(),
            time_control: TimeControl::default(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            search_handle: None,
            result_sender: tx,
            result_receiver: rx,
        }
    }

    /// Run the main UCI loop
    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        println!("id name M4K Chess Engine");
        println!("id author Your Name");
        println!("uciok");

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let command = line.trim();

            if command.is_empty() {
                continue;
            }

            if let Some(response) = self.handle_command(command) {
                writeln!(stdout, "{}", response).unwrap();
                stdout.flush().unwrap();
            }

            // Check for search result
            if let Ok(result) = self.result_receiver.try_recv() {
                if let Some(mv) = result.best_move {
                    writeln!(stdout, "bestmove {}", mv).unwrap();
                    stdout.flush().unwrap();
                }
            }

            if command == "quit" {
                if let Some(handle) = self.search_handle.take() {
                    self.stop_flag.store(true, Ordering::Relaxed);
                    let _ = handle.join();
                }
                break;
            }
        }
    }

    /// Handle a UCI command
    fn handle_command(&mut self, command: &str) -> Option<String> {
        match parse_command(command) {
            Some(UciCommand::Uci) => Some("uciok".to_string()),
            Some(UciCommand::IsReady) => Some("readyok".to_string()),
            Some(UciCommand::NewGame) => {
                self.position.set_startpos();
                Some("readyok".to_string())
            }
            Some(UciCommand::Position { fen, moves }) => {
                self.handle_position(fen, moves);
                None
            }
            Some(UciCommand::Go { time_control }) => {
                self.time_control = time_control;
                self.start_search();
                None
            }
            Some(UciCommand::Stop) => {
                self.stop_flag.store(true, Ordering::Relaxed);
                None
            }
            Some(UciCommand::Quit) => None,
            None => Some(format!("info string Unknown command: {}", command.split_whitespace().next().unwrap_or(""))),
        }
    }

    /// Handle position command
    fn handle_position(&mut self, fen: String, moves: Vec<Move>) {
        if fen == "startpos" {
            self.position.set_startpos();
        } else {
            if let Err(e) = self.position.set_fen(&fen) {
                eprintln!("info string Invalid FEN: {}", e);
                self.position.set_startpos();
                return;
            }
        }

        // Apply moves
        for mv in moves {
            self.position.make_move(mv);
        }
    }

    /// Start search in a separate thread
    fn start_search(&mut self) {
        self.stop_flag.store(false, Ordering::Relaxed);
        let stop_flag_clone = Arc::clone(&self.stop_flag);
        let position = self.position.clone();
        let evaluator = Evaluator::new(); // Create new evaluator
        let mut tt = TranspositionTable::new(); // Create new TT
        let time_control = self.time_control.clone();
        let sender = self.result_sender.clone();

        self.search_handle = Some(thread::spawn(move || {
            let result = iterative_deepening(&time_control, position.side_to_move, &mut tt, &evaluator, &position, &stop_flag_clone);
            let _ = sender.send(result);
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uci_engine_creation() {
        let engine = UciEngine::new();
        // Test that engine can be created
        assert!(true);
    }
}
