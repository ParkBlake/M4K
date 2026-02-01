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

/// UCI Engine state
pub struct UciEngine {
    position: Position,
    evaluator: Evaluator,
    tt: TranspositionTable,
    time_control: TimeControl,
}

impl UciEngine {
    /// Create a new UCI engine
    pub fn new() -> Self {
        // Initialize magic bitboards (must be done once at startup)
        crate::bitboard::magic::init_magics();

        let mut position = Position::empty();
        position.set_startpos();
        UciEngine {
            position,
            evaluator: Evaluator::new(),
            tt: TranspositionTable::new(),
            time_control: TimeControl::default(),
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

            if command == "quit" {
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
                let result = self.search();
                Some(format!("bestmove {}", result.best_move.unwrap()))
            }
            Some(UciCommand::Stop) => {
                // TODO: Implement search stopping
                Some("info string stop not implemented".to_string())
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

    /// Perform search
    fn search(&mut self) -> SearchResult {
        // Use iterative deepening with the current position
        iterative_deepening(
            &self.time_control,
            self.position.side_to_move,
            &mut self.tt,
            &self.evaluator,
            &self.position,
        )
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
