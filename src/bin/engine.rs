//! Main binary entry point for the M4K Chess Engine
//!
//! This implements the UCI (Universal Chess Interface) protocol for communication
//! with chess GUIs and other engines.

use m4k::UciEngine;

fn main() {
    // Initialize the UCI engine
    let mut engine = UciEngine::new();

    // Run the main UCI loop
    engine.run();
}
