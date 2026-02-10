use std::fmt::{self, Display, Formatter};

/// A position within the 8x8 galaxy (quadrant coordinates).
/// Values range 1-8. (1,1) is upper-left, (8,8) is lower-right.
/// X increases left-to-right, Y increases top-to-bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadrantPosition {
    pub x: i32,
    pub y: i32,
}

impl Display for QuadrantPosition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

/// A position within an 8x8 sector grid.
/// Values range 1-8. (1,1) is upper-left, (8,8) is lower-right.
/// X increases left-to-right, Y increases top-to-bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectorPosition {
    pub x: i32,
    pub y: i32,
}

impl Display for SectorPosition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}
