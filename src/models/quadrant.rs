/// Persistent data about a single quadrant in the galaxy.
/// Stores only counts — sector positions are not preserved between visits.
#[derive(Debug, Clone, Copy)]
pub struct QuadrantData {
    pub klingons: i32,
    pub starbases: i32,
    pub stars: i32,
}

impl QuadrantData {
    /// The 3-digit encoded value: klingons*100 + starbases*10 + stars.
    pub fn encoded(&self) -> i32 {
        self.klingons * 100 + self.starbases * 10 + self.stars
    }
}
