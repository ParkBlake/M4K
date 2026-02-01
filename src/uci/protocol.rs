//! UCI Protocol implementation
//!
//! This module handles the Universal Chess Interface protocol,
//! parsing commands from GUIs and sending responses.

use crate::bitboard::position::Position;
use crate::eval::Evaluator;
use crate::movegen::Move;
use crate::search::alphabeta::{iterative_deepening, SearchResult};
use crate::search::transposition::TranspositionTable;
use std::io::{self, BufRead, Write};

/// UCI Engine state
pub struct UciEngine {
    position: Position,
    evaluator: Evaluator,
    tt: TranspositionTable,
}

impl UciEngine {
    /// Create a new UCI engine
    pub fn new() -> Self {
        let mut position = Position::empty();
        position.set_startpos();
        UciEngine {
            position,
            evaluator: Evaluator::new(),
            tt: TranspositionTable::new(),
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
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts[0] {
            "uci" => Some("uciok".to_string()),
            "isready" => Some("readyok".to_string()),
            "ucinewgame" => {
                self.position.set_startpos();
                Some("readyok".to_string())
            }
            "position" => {
                self.parse_position(&parts[1..]);
                None
            }
            "go" => {
                let result = self.search();
                Some(format!("bestmove {}", result.best_move.unwrap()))
            }
            "d" | "display" => {
                // Display current position as FEN and board
                Some(format!(
                    "info string FEN: {}\ninfo string Board:\n{:?}",
                    self.position.to_fen(),
                    self.position
                ))
            }
            "fen" => {
                // Output current FEN
                Some(format!("info string FEN: {}", self.position.to_fen()))
            }
            "quit" => None,
            _ => Some(format!("info string Unknown command: {}", parts[0])),
        }
    }

    /// Parse position command
    fn parse_position(&mut self, args: &[&str]) {
        if args.is_empty() {
            return;
        }
        if args[0] == "startpos" {
            self.position.set_startpos();
            if let Some(moves_idx) = args.iter().position(|&x| x == "moves") {
                for mv_str in &args[moves_idx + 1..] {
                    // TODO: parse and apply moves to self.position
                }
            }
        } else if args[0] == "fen" {
            // Parse FEN string (everything after "fen" up to "moves" or end)
            let moves_idx = args.iter().position(|&x| x == "moves");
            let fen_end = moves_idx.unwrap_or(args.len());
            let fen_str = args[1..fen_end].join(" ");
            if let Err(e) = self.position.set_fen(&fen_str) {
                eprintln!("info string Invalid FEN: {}", e);
                self.position.set_startpos();
            }
            // Handle moves after "moves"
            if let Some(moves_idx) = moves_idx {
                for mv_str in &args[moves_idx + 1..] {
                    // TODO: parse and apply moves to self.position
                }
            }
        }
    }

    /// Perform search
    fn search(&mut self) -> SearchResult {
        // Use iterative deepening with the current position
        let max_depth = 4; // This can be made configurable
        iterative_deepening(
            max_depth,
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
