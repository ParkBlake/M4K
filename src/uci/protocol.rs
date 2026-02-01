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

        println!("id name PI5 Chess Engine");
        println!("id author Your Name");
        println!("uciok");

        for line in stdin.lines() {
            let line = line.unwrap();
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
                // Reset engine state for new game
                Some("readyok".to_string())
            }
            "position" => {
                // Parse position command
                self.parse_position(&parts[1..]);
                None
            }
            "go" => {
                // Start search
                let result = self.search();
                Some(format!("bestmove {}", result.best_move.unwrap()))
            }
            "quit" => None,
            _ => None,
        }
    }

    /// Parse position command
    fn parse_position(&mut self, args: &[&str]) {
        // Supports only "startpos" for now, can be extended for FEN and moves
        if args.is_empty() {
            return;
        }
        if args[0] == "startpos" {
            self.position.set_startpos();
            // Handle moves after "startpos"
            if let Some(moves_idx) = args.iter().position(|&x| x == "moves") {
                for mv_str in &args[moves_idx + 1..] {
                    // TODO: parse and apply moves to self.position
                    // This requires a move parser and make_move logic
                }
            }
        }
        // TODO: Add FEN parsing support if needed
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
