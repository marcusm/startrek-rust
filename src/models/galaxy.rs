use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::constants::{Condition, Device, GALAXY_SIZE, INITIAL_ENERGY, MISSION_DURATION, SECTOR_SIZE, SectorContent};
use super::enterprise::Enterprise;
use super::klingon::Klingon;
use super::position::{QuadrantPosition, SectorPosition};
use super::quadrant::QuadrantData;
use super::sector_map::SectorMap;

/// Top-level game state container.
pub struct Galaxy {
    pub stardate: f64,
    pub starting_stardate: f64,
    pub mission_duration: f64,
    /// 8x8 grid of quadrant data. Internal 0-based: quadrants[y-1][x-1].
    pub quadrants: [[QuadrantData; GALAXY_SIZE]; GALAXY_SIZE],
    /// Computer's knowledge of the galaxy. Starts all zeros, populated by LRS.
    pub computer_memory: [[i32; GALAXY_SIZE]; GALAXY_SIZE],
    pub total_klingons: i32,
    pub total_starbases: i32,
    pub initial_klingons: i32,
    pub enterprise: Enterprise,
    pub sector_map: SectorMap,
    pub rng: StdRng,
}

impl Galaxy {
    /// Create and initialize a new game from the player's seed number.
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Starting stardate (spec 3.2): floor(random * 20 + 20) * 100
        let starting_stardate = (rng.gen::<f64>() * 20.0 + 20.0).floor() * 100.0;

        // Generate galaxy with regeneration guard (spec 3.4, 3.5)
        let (quadrants, total_klingons, total_starbases) = Self::generate_galaxy(&mut rng);

        // Random starting position (spec 3.3)
        let quadrant = QuadrantPosition {
            x: rng.gen_range(1..=8),
            y: rng.gen_range(1..=8),
        };
        let sector = SectorPosition {
            x: rng.gen_range(1..=8),
            y: rng.gen_range(1..=8),
        };

        let mut galaxy = Galaxy {
            stardate: starting_stardate,
            starting_stardate,
            mission_duration: MISSION_DURATION,
            quadrants,
            computer_memory: [[0; GALAXY_SIZE]; GALAXY_SIZE],
            total_klingons,
            total_starbases,
            initial_klingons: total_klingons,
            enterprise: Enterprise::new(quadrant, sector),
            sector_map: SectorMap::new(),
            rng,
        };

        // Enter the starting quadrant (populates sector map)
        galaxy.enter_quadrant();

        galaxy
    }

    /// Generate the 8x8 galaxy. Loops until the regeneration guard passes
    /// (total_klingons > 0 AND total_starbases > 0).
    fn generate_galaxy(
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

    /// Enter the current quadrant: clear sector map and place all entities.
    /// Called on game start and every quadrant transition (spec section 4).
    pub fn enter_quadrant(&mut self) {
        self.sector_map = SectorMap::new();

        // Place the Enterprise
        self.sector_map
            .set(self.enterprise.sector, SectorContent::Enterprise);

        // Place Klingons (each with shields = 200)
        let qdata = self.get_current_quadrant_data();
        let num_klingons = qdata.klingons;
        let num_starbases = qdata.starbases;
        let num_stars = qdata.stars;

        for _ in 0..num_klingons {
            let pos = self.find_random_empty_sector();
            self.sector_map.set(pos, SectorContent::Klingon);
            self.sector_map.klingons.push(Klingon::new(pos));
        }

        // Place starbases
        for _ in 0..num_starbases {
            let pos = self.find_random_empty_sector();
            self.sector_map.set(pos, SectorContent::Starbase);
            self.sector_map.starbase = Some(pos);
        }

        // Place stars
        for _ in 0..num_stars {
            let pos = self.find_random_empty_sector();
            self.sector_map.set(pos, SectorContent::Star);
        }
    }

    /// Find a random empty sector by picking random coordinates until one is empty.
    fn find_random_empty_sector(&mut self) -> SectorPosition {
        loop {
            let pos = SectorPosition {
                x: self.rng.gen_range(1..=8),
                y: self.rng.gen_range(1..=8),
            };
            if self.sector_map.is_empty(pos) {
                return pos;
            }
        }
    }

    /// Get the QuadrantData for the Enterprise's current quadrant.
    fn get_current_quadrant_data(&self) -> QuadrantData {
        let q = self.enterprise.quadrant;
        self.quadrants[(q.y - 1) as usize][(q.x - 1) as usize]
    }

    /// Check if the Enterprise is adjacent to a starbase and dock if so.
    /// Returns true if docked (spec section 9.1-9.2).
    pub fn check_docking(&mut self) -> bool {
        if let Some(base) = self.sector_map.starbase {
            let es = self.enterprise.sector;
            if (es.x - base.x).abs() <= 1 && (es.y - base.y).abs() <= 1 {
                self.enterprise.dock();
                println!("SHIELDS DROPPED FOR DOCKING PURPOSES");
                return true;
            }
        }
        false
    }

    /// Evaluate the ship's condition code (spec section 9.4).
    pub fn evaluate_condition(&self) -> Condition {
        // Check docking adjacency (without side effects)
        if let Some(base) = self.sector_map.starbase {
            let es = self.enterprise.sector;
            if (es.x - base.x).abs() <= 1 && (es.y - base.y).abs() <= 1 {
                return Condition::Docked;
            }
        }

        if !self.sector_map.klingons.is_empty() {
            Condition::Red
        } else if self.enterprise.energy < INITIAL_ENERGY * 0.1 {
            Condition::Yellow
        } else {
            Condition::Green
        }
    }

    /// Short Range Sensor Scan — Command 1 (spec section 6.1).
    pub fn short_range_scan(&mut self) {
        self.check_docking();
        let condition = self.evaluate_condition();

        if self.enterprise.is_damaged(Device::ShortRangeSensors) {
            println!("*** SHORT RANGE SENSORS ARE OUT ***");
            return;
        }

        let border = "-=--=--=--=--=--=--=--=-";
        let e = &self.enterprise;
        let status: [String; SECTOR_SIZE] = [
            format!("STARDATE  {}", self.stardate as i32),
            format!("CONDITION {}", condition.label()),
            format!("QUADRANT  {},{}", e.quadrant.x, e.quadrant.y),
            format!("SECTOR    {},{}", e.sector.x, e.sector.y),
            format!("ENERGY    {}", e.energy as i32),
            format!("SHIELDS   {}", e.shields as i32),
            format!("PHOTON TORPEDOES {}", e.torpedoes),
            String::new(),
        ];

        println!("{}", border);
        for y in 1..=SECTOR_SIZE as i32 {
            let row = self.sector_map.render_row(y);
            let idx = (y - 1) as usize;
            if !status[idx].is_empty() {
                println!("{}        {}", row, status[idx]);
            } else {
                println!("{}", row);
            }
        }
        println!("{}", border);
    }
}
