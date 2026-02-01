//! UCI Commands - Parse and handle UCI commands
//!
//! This module provides functions to parse UCI commands and execute them.

use crate::movegen::Move;

/// Parse a UCI command string
pub fn parse_command(command: &str) -> Option<UciCommand> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    match parts.get(0) {
        Some(&"uci") => Some(UciCommand::Uci),
        Some(&"isready") => Some(UciCommand::IsReady),
        Some(&"ucinewgame") => Some(UciCommand::NewGame),
        Some(&"position") => parse_position_command(&parts[1..]),
        Some(&"go") => parse_go_command(&parts[1..]),
        Some(&"stop") => Some(UciCommand::Stop),
        Some(&"quit") => Some(UciCommand::Quit),
        _ => None,
    }
}

/// UCI command types
pub enum UciCommand {
    Uci,
    IsReady,
    NewGame,
    Position { fen: String, moves: Vec<Move> },
    Go { time_control: TimeControl },
    Stop,
    Quit,
}

/// Time control for search
#[derive(Clone, Copy)]
pub struct TimeControl {
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub movetime: Option<u64>,
    pub infinite: bool,
}

impl Default for TimeControl {
    fn default() -> Self {
        TimeControl {
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: Some(4), // Default depth
            nodes: None,
            movetime: None,
            infinite: false,
        }
    }
}

/// Parse position command
fn parse_position_command(args: &[&str]) -> Option<UciCommand> {
    if args.is_empty() {
        return None;
    }

    let mut fen = String::new();
    let mut moves = Vec::new();
    let mut parsing_moves = false;

    if args[0] == "startpos" {
        fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        parsing_moves = true;
    } else if args[0] == "fen" {
        // Collect FEN string until "moves" or end
        let mut fen_parts = Vec::new();
        for &arg in &args[1..] {
            if arg == "moves" {
                parsing_moves = true;
                break;
            }
            fen_parts.push(arg);
        }
        fen = fen_parts.join(" ");
    } else {
        return None;
    }

    // Parse moves if present
    if parsing_moves {
        if let Some(moves_idx) = args.iter().position(|&x| x == "moves") {
            for &mv_str in &args[moves_idx + 1..] {
                if let Some(mv) = parse_uci_move(mv_str) {
                    moves.push(mv);
                }
            }
        }
    }

    Some(UciCommand::Position { fen, moves })
}

/// Parse a UCI move string into a Move
fn parse_uci_move(mv_str: &str) -> Option<Move> {
    use crate::bitboard::{Piece, Square};

    if mv_str.len() < 4 {
        return None;
    }

    let bytes = mv_str.as_bytes();
    let from_file = (bytes[0] as char as u32).checked_sub('a' as u32)?;
    let from_rank = (bytes[1] as char as u32).checked_sub('1' as u32)?;
    let to_file = (bytes[2] as char as u32).checked_sub('a' as u32)?;
    let to_rank = (bytes[3] as char as u32).checked_sub('1' as u32)?;

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return None;
    }

    let from = Square::new(from_file as u8, from_rank as u8);
    let to = Square::new(to_file as u8, to_rank as u8);

    // Check for promotion
    if mv_str.len() == 5 {
        let promo_char = bytes[4] as char;
        let promo_piece = match promo_char.to_ascii_lowercase() {
            'q' => Piece::Queen,
            'r' => Piece::Rook,
            'b' => Piece::Bishop,
            'n' => Piece::Knight,
            _ => return None,
        };
        Some(Move::promotion(from, to, promo_piece))
    } else {
        Some(Move::new(from, to))
    }
}

/// Parse go command
fn parse_go_command(args: &[&str]) -> Option<UciCommand> {
    let mut time_control = TimeControl {
        wtime: None,
        btime: None,
        winc: None,
        binc: None,
        movestogo: None,
        depth: None,
        nodes: None,
        movetime: None,
        infinite: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "infinite" => {
                time_control.infinite = true;
                i += 1;
            }
            "wtime" => {
                if i + 1 < args.len() {
                    time_control.wtime = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "btime" => {
                if i + 1 < args.len() {
                    time_control.btime = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "winc" => {
                if i + 1 < args.len() {
                    time_control.winc = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "binc" => {
                if i + 1 < args.len() {
                    time_control.binc = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "movestogo" => {
                if i + 1 < args.len() {
                    time_control.movestogo = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "depth" => {
                if i + 1 < args.len() {
                    time_control.depth = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "nodes" => {
                if i + 1 < args.len() {
                    time_control.nodes = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "movetime" => {
                if i + 1 < args.len() {
                    time_control.movetime = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    Some(UciCommand::Go { time_control })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uci_command() {
        assert!(matches!(parse_command("uci"), Some(UciCommand::Uci)));
        assert!(matches!(
            parse_command("isready"),
            Some(UciCommand::IsReady)
        ));
        assert!(matches!(parse_command("quit"), Some(UciCommand::Quit)));
    }
}
