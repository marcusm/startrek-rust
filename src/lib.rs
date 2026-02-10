//! Star Trek Game Engine
//!
//! A Rust implementation of the classic 1978 Super Star Trek game.
//!
//! # Overview
//!
//! This library provides a complete game engine for playing Star Trek.
//! The player commands the USS Enterprise on a mission to destroy all
//! Klingon battle cruisers in the galaxy before time runs out.
//!
//! # Modules
//!
//! - [`game_engine`] - Game state machine and game-over logic
//! - [`models`] - Domain models (Galaxy, Enterprise, Klingon, etc.)
//! - [`services`] - Game services (combat, navigation, scanning, etc.)
//! - [`io`] - Input/output abstractions for testing
//! - [`ui`] - User interface and presentation logic
//!
//! # Example
//!
//! ```rust,no_run
//! use startrek::GameEngine;
//!
//! let mut engine = GameEngine::new(42);
//! // Game logic here
//! ```

pub mod game_engine;
pub mod models;
pub mod services;
pub mod io;
pub mod ui;
pub mod cli;

// Re-export commonly used types
pub use game_engine::{GameEngine, GameState, DefeatReason};
