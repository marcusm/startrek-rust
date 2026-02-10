//! Combat system
//!
//! Handles all combat operations including phaser fire, torpedo launch,
//! shield control, and Klingon attacks.

mod phasers;
mod torpedoes;
mod shields;
mod klingon_attack;

// Re-export public functions
pub use phasers::fire_phasers;
pub use torpedoes::fire_torpedoes;
pub use shields::shield_control;
pub use klingon_attack::{klingons_fire, dead_in_space_loop};

// Re-export helper functions for tests
#[cfg(test)]
pub use phasers::calculate_distance;
