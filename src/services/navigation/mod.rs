//! Navigation system
//!
//! Handles ship movement, course plotting, warp travel,
//! and device damage/repair during navigation.

mod course;
mod movement;
mod damage;

// Re-export main navigation function
pub use movement::navigate;

// Re-export calculate_direction for use by combat module
pub use course::calculate_direction;
