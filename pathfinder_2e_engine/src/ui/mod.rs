//! # Terminal UI Layer
//!
//! Provides text-based display and input handling for the Pathfinder 2e engine.
//! Uses only `std::io` for all I/O -- no external TUI crates required.
//!
//! - `display` -- grid rendering, status panels, and combat log output.
//! - `input` -- player action prompts and direction/target selection.

pub mod display;
pub mod input;
