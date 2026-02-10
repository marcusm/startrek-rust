use rand::rngs::StdRng;
use rand::Rng;

use crate::models::constants::{Device, SectorContent};
use crate::models::enterprise::Enterprise;
use crate::models::klingon::Klingon;
use crate::models::position::SectorPosition;
use crate::models::quadrant::QuadrantData;
use crate::models::sector_map::SectorMap;

/// Enter the current quadrant: clear sector map and place all entities.
/// Called on game start and every quadrant transition (spec section 4).
pub fn enter_quadrant(
    sector_map: &mut SectorMap,
    enterprise: &Enterprise,
    quadrants: &[[QuadrantData; 8]; 8],
    rng: &mut StdRng,
) {
    *sector_map = SectorMap::new();

    // Place the Enterprise
    sector_map.set(enterprise.sector(), SectorContent::Enterprise);

    // Place Klingons (each with shields = 200)
    let q = enterprise.quadrant();
    let qdata = quadrants[(q.y - 1) as usize][(q.x - 1) as usize];
    let num_klingons = qdata.klingons;
    let num_starbases = qdata.starbases;
    let num_stars = qdata.stars;

    for _ in 0..num_klingons {
        let pos = find_random_empty_sector(sector_map, rng);
        sector_map.set(pos, SectorContent::Klingon);
        sector_map.klingons.push(Klingon::new(pos));
    }

    // Place starbases
    for _ in 0..num_starbases {
        let pos = find_random_empty_sector(sector_map, rng);
        sector_map.set(pos, SectorContent::Starbase);
        sector_map.starbase = Some(pos);
    }

    // Place stars
    for _ in 0..num_stars {
        let pos = find_random_empty_sector(sector_map, rng);
        sector_map.set(pos, SectorContent::Star);
    }

    // Red alert check (spec section 4.2)
    if !sector_map.klingons.is_empty() && enterprise.shields() <= 200.0 {
        println!("COMBAT AREA      CONDITION RED");
        println!("   SHIELDS DANGEROUSLY LOW");
    }
}

/// Find a random empty sector by picking random coordinates until one is empty.
fn find_random_empty_sector(sector_map: &SectorMap, rng: &mut StdRng) -> SectorPosition {
    loop {
        let pos = SectorPosition {
            x: rng.gen_range(1..=8),
            y: rng.gen_range(1..=8),
        };
        if sector_map.is_empty(pos) {
            return pos;
        }
    }
}

/// Record a quadrant's data into computer memory.
/// Does nothing if the Computer device is damaged or coordinates are out of range.
pub fn record_quadrant_to_memory(
    computer_memory: &mut [[Option<QuadrantData>; 8]; 8],
    quadrants: &[[QuadrantData; 8]; 8],
    enterprise: &Enterprise,
    x: i32,
    y: i32,
) {
    if enterprise.is_damaged(Device::Computer) {
        return;
    }
    if x >= 1 && x <= 8 && y >= 1 && y <= 8 {
        computer_memory[(y - 1) as usize][(x - 1) as usize] =
            Some(quadrants[(y - 1) as usize][(x - 1) as usize]);
    }
}

/// Update the quadrant's klingon count after removing one.
pub fn decrement_quadrant_klingons(
    quadrants: &mut [[QuadrantData; 8]; 8],
    enterprise: &Enterprise,
) {
    let q = enterprise.quadrant();
    quadrants[(q.y - 1) as usize][(q.x - 1) as usize].klingons -= 1;
}

/// Update the quadrant's starbase count after removing one.
pub fn decrement_quadrant_starbases(
    quadrants: &mut [[QuadrantData; 8]; 8],
    enterprise: &Enterprise,
) {
    let q = enterprise.quadrant();
    quadrants[(q.y - 1) as usize][(q.x - 1) as usize].starbases -= 1;
}
