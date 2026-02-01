//! Transposition table for search optimization
//!
//! This module implements a transposition table to cache search results
//! and avoid redundant computation.

use crate::movegen::Move;

/// Entry in the transposition table
#[derive(Clone, Copy)]
pub struct TTEntry {
    pub score: i32,
    pub best_move: Move,
    pub depth: i32,
    pub node_type: NodeType,
}

/// Type of node stored in the transposition table
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact,  // Exact score
    Lower,  // Lower bound (fail high)
    Upper,  // Upper bound (fail low)
}

/// Transposition table using a simple hash map
pub struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
    size: usize,
}

impl TranspositionTable {
    /// Create a new transposition table with the given size in MB
    pub fn new() -> Self {
        let size = 16 * 1024 * 1024; // 16MB default
        let num_entries = size / std::mem::size_of::<Option<TTEntry>>();
        Self {
            table: vec![None; num_entries],
            size: num_entries,
        }
    }

    /// Create a new transposition table with custom size in MB
    pub fn with_size(size_mb: usize) -> Self {
        let size_bytes = size_mb * 1024 * 1024;
        let num_entries = size_bytes / std::mem::size_of::<Option<TTEntry>>();
        Self {
            table: vec![None; num_entries],
            size: num_entries,
        }
    }

    /// Compute hash index for a position
    fn hash_index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }

    /// Probe the transposition table for a position
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let index = self.hash_index(hash);
        self.table[index]
    }

    /// Store an entry in the transposition table
    pub fn store(&mut self, hash: u64, entry: TTEntry) {
        let index = self.hash_index(hash);
        self.table[index] = Some(entry);
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        for entry in &mut self.table {
            *entry = None;
        }
    }

    /// Get the number of entries in the table
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get statistics about table usage
    pub fn stats(&self) -> TTStats {
        let mut used = 0;
        for entry in &self.table {
            if entry.is_some() {
                used += 1;
            }
        }

        TTStats {
            total_entries: self.size,
            used_entries: used,
            usage_percent: (used as f32 / self.size as f32) * 100.0,
        }
    }
}

/// Statistics about transposition table usage
pub struct TTStats {
    pub total_entries: usize,
    pub used_entries: usize,
    pub usage_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::Square;

    #[test]
    fn test_tt_creation() {
        let tt = TranspositionTable::new();
        assert!(tt.size() > 0);
    }

    #[test]
    fn test_tt_store_probe() {
        let mut tt = TranspositionTable::new();
        let hash = 12345u64;
        let entry = TTEntry {
            score: 100,
            best_move: Move::new(Square::E2, Square::E4),
            depth: 5,
            node_type: NodeType::Exact,
        };

        tt.store(hash, entry);
        let retrieved = tt.probe(hash);

        assert!(retrieved.is_some());
        let retrieved_entry = retrieved.unwrap();
        assert_eq!(retrieved_entry.score, 100);
        assert_eq!(retrieved_entry.depth, 5);
    }

    #[test]
    fn test_tt_clear() {
        let mut tt = TranspositionTable::new();
        let hash = 12345u64;
        let entry = TTEntry {
            score: 100,
            best_move: Move::new(Square::E2, Square::E4),
            depth: 5,
            node_type: NodeType::Exact,
        };

        tt.store(hash, entry);
        assert!(tt.probe(hash).is_some());

        tt.clear();
        assert!(tt.probe(hash).is_none());
    }
}
