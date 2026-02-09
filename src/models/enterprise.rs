use super::constants::{Device, INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES, NUM_DEVICES};
use super::position::{QuadrantPosition, SectorPosition};

/// The player's starship.
pub struct Enterprise {
    pub quadrant: QuadrantPosition,
    pub sector: SectorPosition,
    pub energy: f64,
    pub torpedoes: i32,
    pub shields: f64,
    /// Damage state for each of the 8 devices.
    /// 0 = operational, negative = damaged, positive = improved.
    pub devices: [f64; NUM_DEVICES],
}

impl Enterprise {
    pub fn new(quadrant: QuadrantPosition, sector: SectorPosition) -> Self {
        Enterprise {
            quadrant,
            sector,
            energy: INITIAL_ENERGY,
            torpedoes: INITIAL_TORPEDOES,
            shields: INITIAL_SHIELDS,
            devices: [0.0; NUM_DEVICES],
        }
    }

    pub fn is_damaged(&self, device: Device) -> bool {
        self.devices[device as usize] < 0.0
    }

    /// Reset ship resources when docking at a starbase (spec section 9.2).
    pub fn dock(&mut self) {
        self.energy = INITIAL_ENERGY;
        self.torpedoes = INITIAL_TORPEDOES;
        self.shields = INITIAL_SHIELDS;
    }

    /// Check if the Enterprise is adjacent to (or at) a starbase (spec section 9.1).
    pub fn is_adjacent_to_starbase(&self, starbase: Option<SectorPosition>) -> bool {
        if let Some(base) = starbase {
            (self.sector.x - base.x).abs() <= 1 && (self.sector.y - base.y).abs() <= 1
        } else {
            false
        }
    }

    /// Check if the Enterprise is adjacent to a starbase and dock if so.
    /// Returns true if docked (spec section 9.1-9.2).
    pub fn check_docking(&mut self, starbase: Option<SectorPosition>) -> bool {
        if self.is_adjacent_to_starbase(starbase) {
            self.dock();
            println!("SHIELDS DROPPED FOR DOCKING PURPOSES");
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::{INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES};
    use crate::models::position::SectorPosition;

    /// Helper: create an Enterprise with reduced resources at a given sector.
    fn enterprise_at(sector: SectorPosition) -> Enterprise {
        let mut e = Enterprise::new(
            QuadrantPosition { x: 1, y: 1 },
            sector,
        );
        e.energy = 1000.0;
        e.shields = 500.0;
        e.torpedoes = 3;
        e
    }

    #[test]
    fn docking_when_adjacent_horizontally() {
        let mut e = enterprise_at(SectorPosition { x: 4, y: 4 });
        let starbase = Some(SectorPosition { x: 5, y: 4 });

        assert!(e.check_docking(starbase));
        assert_eq!(e.energy, INITIAL_ENERGY);
        assert_eq!(e.torpedoes, INITIAL_TORPEDOES);
        assert_eq!(e.shields, INITIAL_SHIELDS);
    }

    #[test]
    fn docking_when_adjacent_diagonally() {
        let mut e = enterprise_at(SectorPosition { x: 3, y: 3 });
        let starbase = Some(SectorPosition { x: 4, y: 4 });

        assert!(e.check_docking(starbase));
    }

    #[test]
    fn no_docking_when_too_far() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        let starbase = Some(SectorPosition { x: 4, y: 4 });

        assert!(!e.check_docking(starbase));
        assert_eq!(e.energy, 1000.0);
        assert_eq!(e.torpedoes, 3);
    }

    #[test]
    fn no_docking_when_no_starbase() {
        let mut e = enterprise_at(SectorPosition { x: 4, y: 4 });

        assert!(!e.check_docking(None));
    }

    #[test]
    fn docking_when_distance_exactly_one() {
        let base = SectorPosition { x: 4, y: 4 };
        let adjacent_positions = [
            SectorPosition { x: 3, y: 3 },
            SectorPosition { x: 4, y: 3 },
            SectorPosition { x: 5, y: 3 },
            SectorPosition { x: 3, y: 4 },
            SectorPosition { x: 5, y: 4 },
            SectorPosition { x: 3, y: 5 },
            SectorPosition { x: 4, y: 5 },
            SectorPosition { x: 5, y: 5 },
        ];
        for pos in &adjacent_positions {
            let mut e = enterprise_at(*pos);
            assert!(
                e.check_docking(Some(base)),
                "should dock at ({}, {}) next to base at (4, 4)",
                pos.x,
                pos.y
            );
        }
    }
}
