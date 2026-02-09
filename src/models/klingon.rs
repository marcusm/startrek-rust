use super::constants::KLINGON_INITIAL_SHIELDS;
use super::position::SectorPosition;

/// A Klingon warship within a quadrant's sector grid.
/// Up to 3 per quadrant.
#[derive(Debug, Clone, Copy)]
pub struct Klingon {
    pub sector: SectorPosition,
    pub shields: f64,
}

impl Klingon {
    pub fn new(sector: SectorPosition) -> Self {
        Klingon {
            sector,
            shields: KLINGON_INITIAL_SHIELDS,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.shields > 0.0
    }
}
