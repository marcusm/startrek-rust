use crate::models::galaxy::Galaxy;

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
    pub fn new(seed: u64) -> Self {
        Self {
            galaxy: Galaxy::new(seed),
            state: GameState::Playing,
        }
    }

    pub fn galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    pub fn galaxy_mut(&mut self) -> &mut Galaxy {
        &mut self.galaxy
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Check for game over conditions and update state
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
