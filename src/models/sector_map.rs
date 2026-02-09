use super::constants::{SectorContent, SECTOR_SIZE, MAX_KLINGONS_PER_QUADRANT};
use super::klingon::Klingon;
use super::position::SectorPosition;

/// The 8x8 sector grid for the current quadrant.
/// Regenerated every time the Enterprise enters a quadrant.
pub struct SectorMap {
    /// 8x8 grid of sector contents. Internal 0-based indexing: grid[y-1][x-1].
    grid: [[SectorContent; SECTOR_SIZE]; SECTOR_SIZE],
    /// Active Klingons in this quadrant (up to 3).
    pub klingons: Vec<Klingon>,
    /// Position of the starbase in this quadrant, if any.
    pub starbase: Option<SectorPosition>,
}

impl SectorMap {
    pub fn new() -> Self {
        SectorMap {
            grid: [[SectorContent::Empty; SECTOR_SIZE]; SECTOR_SIZE],
            klingons: Vec::with_capacity(MAX_KLINGONS_PER_QUADRANT),
            starbase: None,
        }
    }

    /// Get the content at a 1-based sector position.
    pub fn get(&self, pos: SectorPosition) -> SectorContent {
        self.grid[(pos.y - 1) as usize][(pos.x - 1) as usize]
    }

    /// Set the content at a 1-based sector position.
    pub fn set(&mut self, pos: SectorPosition, content: SectorContent) {
        self.grid[(pos.y - 1) as usize][(pos.x - 1) as usize] = content;
    }

    /// Check if a 1-based sector position is empty.
    pub fn is_empty(&self, pos: SectorPosition) -> bool {
        self.get(pos) == SectorContent::Empty
    }
}
