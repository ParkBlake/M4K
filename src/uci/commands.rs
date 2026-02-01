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

/// Parse position command
fn parse_position_command(args: &[&str]) -> Option<UciCommand> {
    // Placeholder - would parse FEN and moves
    Some(UciCommand::Position {
        fen: "startpos".to_string(),
        moves: Vec::new(),
    })
}

/// Parse go command
fn parse_go_command(args: &[&str]) -> Option<UciCommand> {
    // Placeholder - would parse time controls
    Some(UciCommand::Go {
        time_control: TimeControl {
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: None,
            nodes: None,
            movetime: None,
            infinite: false,
        },
    })
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
