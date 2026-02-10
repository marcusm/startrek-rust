use crate::models::constants::COURSE_VECTORS;
use crate::models::position::{QuadrantPosition, SectorPosition};

/// Calculate the direction vector for a given course value (1.0 ..< 9.0).
/// Uses linear interpolation between adjacent integer course vectors.
pub fn calculate_direction(course: f64) -> (f64, f64) {
    let r = course.floor() as usize;
    let frac = course - course.floor();
    let dx = COURSE_VECTORS[r].0 + (COURSE_VECTORS[r + 1].0 - COURSE_VECTORS[r].0) * frac;
    let dy = COURSE_VECTORS[r].1 + (COURSE_VECTORS[r + 1].1 - COURSE_VECTORS[r].1) * frac;
    (dx, dy)
}

/// Calculate the new quadrant and sector position after a quadrant boundary
/// crossing. Uses absolute galactic coordinates with sector-zero correction
/// and galaxy-edge clamping.
pub fn calculate_quadrant_crossing(
    quad_x: i32,
    quad_y: i32,
    sect_x: i32,
    sect_y: i32,
    dx: f64,
    dy: f64,
    n: i32,
) -> (QuadrantPosition, SectorPosition) {
    let abs_x = quad_x as f64 * 8.0 + sect_x as f64 + dx * n as f64;
    let abs_y = quad_y as f64 * 8.0 + sect_y as f64 + dy * n as f64;

    let mut new_quad_x = (abs_x / 8.0).floor() as i32;
    let mut new_quad_y = (abs_y / 8.0).floor() as i32;
    let mut new_sect_x = (abs_x - new_quad_x as f64 * 8.0 + 0.5).floor() as i32;
    let mut new_sect_y = (abs_y - new_quad_y as f64 * 8.0 + 0.5).floor() as i32;

    // Sector-zero correction
    if new_sect_x == 0 {
        new_quad_x -= 1;
        new_sect_x = 8;
    }
    if new_sect_y == 0 {
        new_quad_y -= 1;
        new_sect_y = 8;
    }

    // Clamp quadrant to galaxy boundaries (1-8)
    new_quad_x = new_quad_x.clamp(1, 8);
    new_quad_y = new_quad_y.clamp(1, 8);

    // Clamp sector to valid range (1-8) in case of edge effects
    new_sect_x = new_sect_x.clamp(1, 8);
    new_sect_y = new_sect_y.clamp(1, 8);

    (
        QuadrantPosition {
            x: new_quad_x,
            y: new_quad_y,
        },
        SectorPosition {
            x: new_sect_x,
            y: new_sect_y,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Direction vector tests ---

    #[test]
    fn direction_integer_courses() {
        let cases = [
            (1.0, (1.0, 0.0)),   // east
            (2.0, (1.0, -1.0)),  // northeast
            (3.0, (0.0, -1.0)),  // north
            (4.0, (-1.0, -1.0)), // northwest
            (5.0, (-1.0, 0.0)),  // west
            (6.0, (-1.0, 1.0)),  // southwest
            (7.0, (0.0, 1.0)),   // south
            (8.0, (1.0, 1.0)),   // southeast
        ];
        for (course, (expected_dx, expected_dy)) in &cases {
            let (dx, dy) = calculate_direction(*course);
            assert!(
                (dx - expected_dx).abs() < 1e-10 && (dy - expected_dy).abs() < 1e-10,
                "course {} expected ({}, {}), got ({}, {})",
                course,
                expected_dx,
                expected_dy,
                dx,
                dy,
            );
        }
    }

    #[test]
    fn direction_fractional_interpolation() {
        // Course 1.5: midpoint between course 1 (1,0) and course 2 (1,-1) → (1.0, -0.5)
        let (dx, dy) = calculate_direction(1.5);
        assert!((dx - 1.0).abs() < 1e-10);
        assert!((dy - (-0.5)).abs() < 1e-10);

        // Course 4.5: midpoint between course 4 (-1,-1) and course 5 (-1,0) → (-1.0, -0.5)
        let (dx, dy) = calculate_direction(4.5);
        assert!((dx - (-1.0)).abs() < 1e-10);
        assert!((dy - (-0.5)).abs() < 1e-10);
    }

    // --- Quadrant crossing tests ---

    #[test]
    fn quadrant_crossing_basic_east() {
        // Quadrant (1,1), sector (8,4), moving east (dx=1, dy=0), 8 steps
        let (quad, sect) = calculate_quadrant_crossing(1, 1, 8, 4, 1.0, 0.0, 8);
        assert_eq!(quad.x, 2, "should move to quadrant 2");
        assert_eq!(quad.y, 1, "y quadrant unchanged");
        // abs_x = 1*8 + 8 + 1*8 = 24, new_quad_x = floor(24/8) = 3,
        // new_sect_x = floor(24 - 3*8 + 0.5) = floor(0.5) = 0 → sector-zero correction
        // → quad_x = 2, sect_x = 8
        assert_eq!(quad.x, 2);
        assert_eq!(sect.x, 8);
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_west() {
        // Quadrant (1,4), sector (1,4), moving west (dx=-1, dy=0), 8 steps
        // abs_x = 1*8 + 1 + (-1)*8 = 1, new_quad_x = floor(1/8) = 0 → clamp to 1
        let (quad, _sect) = calculate_quadrant_crossing(1, 4, 1, 4, -1.0, 0.0, 8);
        assert_eq!(quad.x, 1, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_north() {
        // Quadrant (4,1), sector (4,1), moving north (dx=0, dy=-1), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(4, 1, 4, 1, 0.0, -1.0, 8);
        assert_eq!(quad.y, 1, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_east() {
        // Quadrant (8,4), sector (8,4), moving east (dx=1, dy=0), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(8, 4, 8, 4, 1.0, 0.0, 8);
        assert_eq!(quad.x, 8, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_south() {
        // Quadrant (4,8), sector (4,8), moving south (dx=0, dy=1), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(4, 8, 4, 8, 0.0, 1.0, 8);
        assert_eq!(quad.y, 8, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_sector_zero_correction() {
        // Set up a scenario where abs coordinate gives sector = 0.
        // Quadrant (2,2), sector (8,8), moving east 8 steps:
        // abs_x = 2*8 + 8 + 1*8 = 32, new_quad_x = floor(32/8) = 4
        // new_sect_x = floor(32 - 4*8 + 0.5) = floor(0.5) = 0
        // Correction: quad_x = 3, sect_x = 8
        let (quad, sect) = calculate_quadrant_crossing(2, 2, 8, 8, 1.0, 0.0, 8);
        assert_eq!(quad.x, 3);
        assert_eq!(sect.x, 8);
    }
}
