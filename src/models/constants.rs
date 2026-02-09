pub const GALAXY_SIZE: usize = 8;
pub const SECTOR_SIZE: usize = 8;
pub const MAX_KLINGONS_PER_QUADRANT: usize = 3;

pub const INITIAL_ENERGY: f64 = 3000.0;
pub const INITIAL_TORPEDOES: i32 = 10;
pub const INITIAL_SHIELDS: f64 = 0.0;
pub const KLINGON_INITIAL_SHIELDS: f64 = 200.0;
pub const MISSION_DURATION: f64 = 30.0;

pub const NUM_DEVICES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    WarpEngines = 0,
    ShortRangeSensors = 1,
    LongRangeSensors = 2,
    PhaserControl = 3,
    PhotonTubes = 4,
    DamageControl = 5,
    ShieldControl = 6,
    Computer = 7,
}

impl Device {
    pub fn name(&self) -> &'static str {
        match self {
            Device::WarpEngines => "WARP ENGINES",
            Device::ShortRangeSensors => "S.R. SENSORS",
            Device::LongRangeSensors => "L.R. SENSORS",
            Device::PhaserControl => "PHASER CNTRL",
            Device::PhotonTubes => "PHOTON TUBES",
            Device::DamageControl => "DAMAGE CNTRL",
            Device::ShieldControl => "SHIELD CNTRL",
            Device::Computer => "COMPUTER",
        }
    }

    pub const ALL: [Device; NUM_DEVICES] = [
        Device::WarpEngines,
        Device::ShortRangeSensors,
        Device::LongRangeSensors,
        Device::PhaserControl,
        Device::PhotonTubes,
        Device::DamageControl,
        Device::ShieldControl,
        Device::Computer,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorContent {
    Empty = 0,
    Enterprise = 1,
    Klingon = 2,
    Starbase = 3,
    Star = 4,
}

impl SectorContent {
    pub fn symbol(&self) -> &'static str {
        match self {
            SectorContent::Empty => "   ",
            SectorContent::Enterprise => "<*>",
            SectorContent::Klingon => "+++",
            SectorContent::Starbase => ">!<",
            SectorContent::Star => " * ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Green,
    Yellow,
    Red,
    Docked,
}

impl Condition {
    pub fn label(&self) -> &'static str {
        match self {
            Condition::Green => "GREEN",
            Condition::Yellow => "YELLOW",
            Condition::Red => "RED",
            Condition::Docked => "DOCKED",
        }
    }
}

/// Course direction vectors for courses 1-9. Index 0 is unused.
/// Format: (delta_x, delta_y).
pub const COURSE_VECTORS: [(f64, f64); 10] = [
    (0.0, 0.0),   // index 0: unused
    (1.0, 0.0),   // course 1
    (1.0, -1.0),  // course 2
    (0.0, -1.0),  // course 3
    (-1.0, -1.0), // course 4
    (-1.0, 0.0),  // course 5
    (-1.0, 1.0),  // course 6
    (0.0, 1.0),   // course 7
    (1.0, 1.0),   // course 8
    (1.0, 0.0),   // course 9 (same as 1, for interpolation)
];
