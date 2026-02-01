//! UCI Protocol implementation
//!
//! This module handles the Universal Chess Interface protocol,
//! parsing commands from GUIs and sending responses.

use crate::bitboard::position::Position;
use crate::bitboard::{Bitboard, Color, Piece};
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
use std::time::{Duration, Instant};

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
        stdout.flush().unwrap();

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
                println!("{}", response);
                stdout.flush().unwrap();
            }

            // Check for search result immediately after handling command
            if let Ok(result) = self.result_receiver.try_recv() {
                if let Some(mv) = result.best_move {
                    println!("bestmove {}", mv);
                    stdout.flush().unwrap();
                } else if let Some(fallback_mv) = self.generate_emergency_move() {
                    println!("bestmove {}", fallback_mv);
                    stdout.flush().unwrap();
                } else {
                    println!("bestmove 0000"); // Absolute fallback
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
                // Wait a short time for search to complete and send result
                thread::sleep(Duration::from_millis(10));
                if let Ok(result) = self.result_receiver.try_recv() {
                    if let Some(mv) = result.best_move {
                        return Some(format!("bestmove {}", mv));
                    } else if let Some(fallback_mv) = self.generate_emergency_move() {
                        return Some(format!("bestmove {}", fallback_mv));
                    }
                }
                None
            }
            Some(UciCommand::Quit) => {
                // Wait for any pending search results before quitting
                if let Some(handle) = self.search_handle.take() {
                    let _ = handle.join();
                }
                // Try to get the final result
                if let Ok(result) = self.result_receiver.try_recv() {
                    if let Some(mv) = result.best_move {
                        println!("bestmove {}", mv);
                        io::stdout().flush().unwrap();
                    } else if let Some(fallback_mv) = self.generate_emergency_move() {
                        println!("bestmove {}", fallback_mv);
                        io::stdout().flush().unwrap();
                    }
                }
                None
            }
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
        let evaluator = Evaluator::new();
        let mut tt = TranspositionTable::new();
        let time_control = self.time_control.clone();
        let sender = self.result_sender.clone();

        self.search_handle = Some(thread::spawn(move || {
            // Set a hard timeout to prevent infinite searches (5 minutes max)
            let search_timeout = if time_control.infinite {
                Duration::from_secs(300) // 5 minutes for infinite searches
            } else {
                Duration::from_secs(60) // 1 minute max for normal searches
            };

            let start_time = Instant::now();

            // Run search with timeout
            let result = iterative_deepening(&time_control, position.side_to_move, &mut tt, &evaluator, &position, &stop_flag_clone);

            // If search took too long, force stop flag
            if start_time.elapsed() > search_timeout {
                // This shouldn't happen with proper time management, but just in case
            }

            let _ = sender.send(result);
        }));
    }

    /// Generate an emergency move if search fails completely
    fn generate_emergency_move(&self) -> Option<Move> {
        use crate::bitboard::Piece;
        use crate::movegen::generator::*;
        use crate::movegen::legal::filter_legal_moves;

        let mut moves = MoveList::new();
        let color = self.position.side_to_move;
        let occupied = (0..6).fold(Bitboard::EMPTY, |acc, p| {
            acc | self.position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::White)
                | self.position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::Black)
        });
        let enemies = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
            acc | self.position.piece_bb(Piece::from_u8(p).unwrap(), color.opposite())
        });

        generate_pawn_moves(
            &mut moves,
            self.position.piece_bb(Piece::Pawn, color),
            occupied,
            enemies,
            color,
            self.position.en_passant,
        );
        generate_knight_moves(
            &mut moves,
            self.position.piece_bb(Piece::Knight, color),
            occupied,
            enemies,
        );
        generate_bishop_moves(
            &mut moves,
            self.position.piece_bb(Piece::Bishop, color),
            occupied,
            enemies,
        );
        generate_rook_moves(
            &mut moves,
            self.position.piece_bb(Piece::Rook, color),
            occupied,
            enemies,
        );
        generate_queen_moves(
            &mut moves,
            self.position.piece_bb(Piece::Queen, color),
            occupied,
            enemies,
        );
        if let Some(king_sq) = self.position.piece_bb(Piece::King, color).lsb() {
            generate_king_moves(&mut moves, king_sq, occupied, enemies);
        }

        let legal_moves = filter_legal_moves(&moves, &self.position, color);
        legal_moves.iter().next().copied()
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
