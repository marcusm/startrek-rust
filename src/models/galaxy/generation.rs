use rand::rngs::StdRng;
use rand::Rng;

use crate::models::constants::GALAXY_SIZE;
use crate::models::quadrant::QuadrantData;

/// Generate the 8x8 galaxy. Loops until the regeneration guard passes
/// (total_klingons > 0 AND total_starbases > 0).
pub fn generate_galaxy(
    rng: &mut StdRng,
) -> ([[QuadrantData; GALAXY_SIZE]; GALAXY_SIZE], i32, i32) {
    loop {
        let mut quadrants = [[QuadrantData {
            klingons: 0,
            starbases: 0,
            stars: 0,
        }; GALAXY_SIZE]; GALAXY_SIZE];
        let mut total_klingons = 0;
        let mut total_starbases = 0;

        // Using indexed loops here because we need both x and y indices for 2D array access
        #[allow(clippy::needless_range_loop)]
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                let f: f64 = rng.gen();
                let klingons = if f > 0.98 {
                    3
                } else if f > 0.95 {
                    2
                } else if f > 0.80 {
                    1
                } else {
                    0
                };

                let f: f64 = rng.gen();
                let starbases = if f > 0.96 { 1 } else { 0 };

                let stars = (rng.gen::<f64>() * 8.0 + 1.0).floor() as i32;

                quadrants[y][x] = QuadrantData {
                    klingons,
                    starbases,
                    stars,
                };
                total_klingons += klingons;
                total_starbases += starbases;
            }
        }

        if total_klingons > 0 && total_starbases > 0 {
            return (quadrants, total_klingons, total_starbases);
        }
    }
}
