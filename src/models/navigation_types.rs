//! Type-safe wrappers for navigation values

use std::fmt;

/// Course direction (1.0 to 9.0)
///
/// Represents navigation course in the game:
/// - 1 = North
/// - 3 = East
/// - 5 = South
/// - 7 = West
/// - 2, 4, 6, 8 = Diagonal directions
/// - Fractional values interpolate between directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Course(f64);

impl Course {
    /// Create a new course value
    ///
    /// # Arguments
    /// * `value` - Course direction (1.0 to 9.0)
    ///
    /// # Returns
    /// Ok(Course) if valid, Err with message if invalid
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if (1.0..=9.0).contains(&value) {
            Ok(Course(value))
        } else {
            Err("Course must be between 1.0 and 9.0")
        }
    }

    /// Get the course value
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Course {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// Warp factor (0.0 to 8.0)
///
/// Represents warp speed:
/// - 0.0 = No movement
/// - 0.1 - 0.9 = Sub-warp (no time advancement)
/// - 1.0 - 8.0 = Full warp speeds
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WarpFactor(f64);

impl WarpFactor {
    /// Create a new warp factor
    ///
    /// # Arguments
    /// * `value` - Warp speed (0.0 to 8.0)
    ///
    /// # Returns
    /// Ok(WarpFactor) if valid, Err with message if invalid
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if (0.0..=8.0).contains(&value) {
            Ok(WarpFactor(value))
        } else {
            Err("Warp factor must be between 0.0 and 8.0")
        }
    }

    /// Get the warp factor value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if this is sub-warp speed (< 1.0)
    #[allow(dead_code)]
    pub fn is_subwarp(&self) -> bool {
        self.0 < 1.0
    }

    /// Check if this is full warp speed (>= 1.0)
    pub fn is_warp(&self) -> bool {
        self.0 >= 1.0
    }
}

impl fmt::Display for WarpFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_valid_range() {
        assert!(Course::new(1.0).is_ok());
        assert!(Course::new(5.5).is_ok());
        assert!(Course::new(9.0).is_ok());
    }

    #[test]
    fn course_invalid_range() {
        assert!(Course::new(0.0).is_err());
        assert!(Course::new(0.5).is_err());
        assert!(Course::new(9.1).is_err());
        assert!(Course::new(10.0).is_err());
    }

    #[test]
    fn warp_valid_range() {
        assert!(WarpFactor::new(0.0).is_ok());
        assert!(WarpFactor::new(4.5).is_ok());
        assert!(WarpFactor::new(8.0).is_ok());
    }

    #[test]
    fn warp_invalid_range() {
        assert!(WarpFactor::new(-0.1).is_err());
        assert!(WarpFactor::new(8.1).is_err());
        assert!(WarpFactor::new(10.0).is_err());
    }

    #[test]
    fn warp_subwarp_check() {
        let sub = WarpFactor::new(0.5).unwrap();
        let full = WarpFactor::new(2.0).unwrap();

        assert!(sub.is_subwarp());
        assert!(!sub.is_warp());

        assert!(!full.is_subwarp());
        assert!(full.is_warp());
    }
}
