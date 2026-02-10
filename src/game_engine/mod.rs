//! Game state machine
//!
//! Manages the overall game state, checking for victory and defeat conditions.
//! The GameEngine owns the Galaxy and tracks whether the game is still being played.

use crate::models::galaxy::Galaxy;

/// Core game engine that manages game state and victory/defeat conditions
pub struct GameEngine {
    galaxy: Galaxy,
    state: GameState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Playing,
    Victory { rating: i32 },
    Defeat { reason: DefeatReason },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefeatReason {
    ShipDestroyed,
    TimeExpired,
    DeadInSpace,
}

impl GameEngine {
    /// Creates a new game engine with a procedurally generated galaxy
    ///
    /// # Arguments
    ///
    /// * `seed` - Random number generator seed for galaxy generation
    ///
    /// # Returns
    ///
    /// A new GameEngine in the Playing state with a freshly generated galaxy
    pub fn new(seed: u64) -> Self {
        Self {
            galaxy: Galaxy::new(seed),
            state: GameState::Playing,
        }
    }

    /// Returns an immutable reference to the galaxy
    pub fn galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    /// Returns a mutable reference to the galaxy
    pub fn galaxy_mut(&mut self) -> &mut Galaxy {
        &mut self.galaxy
    }

    /// Returns the current game state
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Checks for game over conditions and updates the game state
    ///
    /// # Returns
    ///
    /// * `Some(GameState)` if the game has ended (Victory or Defeat)
    /// * `None` if the game is still in progress
    ///
    /// # Victory Conditions
    ///
    /// The player wins when all Klingon battle cruisers are destroyed.
    /// An efficiency rating is calculated based on time remaining and losses.
    ///
    /// # Defeat Conditions
    ///
    /// The player loses if:
    /// - The Enterprise is destroyed (shields fall below 0)
    /// - Time expires before all Klingons are destroyed
    pub fn check_game_over(&mut self) -> Option<GameState> {
        if self.state != GameState::Playing {
            return Some(self.state.clone());
        }

        // Victory: all Klingons destroyed
        if self.galaxy.all_klingons_destroyed() {
            let rating = self.galaxy.efficiency_rating();
            self.state = GameState::Victory { rating };
            return Some(self.state.clone());
        }

        // Defeat: ship destroyed (shields < 0)
        if self.galaxy.enterprise().shields() < 0.0 {
            self.state = GameState::Defeat {
                reason: DefeatReason::ShipDestroyed,
            };
            return Some(self.state.clone());
        }

        // Defeat: time expired
        if self.galaxy.is_time_expired() {
            self.state = GameState::Defeat {
                reason: DefeatReason::TimeExpired,
            };
            return Some(self.state.clone());
        }

        None
    }
}
