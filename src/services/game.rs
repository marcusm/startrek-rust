use crate::models::galaxy::Galaxy;

pub struct Game {
    pub galaxy: Galaxy,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Game {
            galaxy: Galaxy::new(seed),
        }
    }

    pub fn run(&mut self) {
        self.print_mission_briefing();
        // Future: game command loop
    }

    fn print_mission_briefing(&self) {
        let g = &self.galaxy;
        let plural = if g.total_starbases != 1 { "S" } else { "" };
        println!(
            "YOU MUST DESTROY {} KINGONS IN {} STARDATES WITH {} STARBASE{}",
            g.total_klingons, g.mission_duration as i32, g.total_starbases, plural,
        );
    }
}
