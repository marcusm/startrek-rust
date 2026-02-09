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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_all_zeros() {
        let q = QuadrantData { klingons: 0, starbases: 0, stars: 0 };
        assert_eq!(q.encoded(), 0);
    }

    #[test]
    fn encoded_only_klingons() {
        let q = QuadrantData { klingons: 3, starbases: 0, stars: 0 };
        assert_eq!(q.encoded(), 300);
    }

    #[test]
    fn encoded_only_starbases() {
        let q = QuadrantData { klingons: 0, starbases: 1, stars: 0 };
        assert_eq!(q.encoded(), 10);
    }

    #[test]
    fn encoded_only_stars() {
        let q = QuadrantData { klingons: 0, starbases: 0, stars: 5 };
        assert_eq!(q.encoded(), 5);
    }

    #[test]
    fn encoded_mixed() {
        let q = QuadrantData { klingons: 2, starbases: 1, stars: 7 };
        assert_eq!(q.encoded(), 217);
    }

    #[test]
    fn encoded_max_values() {
        let q = QuadrantData { klingons: 3, starbases: 1, stars: 8 };
        assert_eq!(q.encoded(), 318);
    }
}
