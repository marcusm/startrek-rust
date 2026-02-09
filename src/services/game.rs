use std::io::{self, Write};

use crate::models::galaxy::Galaxy;
use crate::services::combat;
use crate::services::computer;
use crate::services::navigation;
use crate::services::scan;

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
        scan::short_range_scan(&mut self.galaxy);

        loop {
            let input = Self::read_line("COMMAND");
            let input = input.trim();

            match input {
                "0" => navigation::navigate(&mut self.galaxy),
                "1" => scan::short_range_scan(&mut self.galaxy),
                "2" => scan::long_range_scan(&mut self.galaxy),
                "3" => combat::fire_phasers(&mut self.galaxy),
                "4" => combat::fire_torpedoes(&mut self.galaxy),
                "5" => combat::shield_control(&mut self.galaxy),
                "6" => self.galaxy.enterprise.damage_report(),
                "7" => computer::library_computer(&mut self.galaxy),
                "q" | "Q" => {
                    println!("GOODBYE, CAPTAIN.");
                    break;
                }
                _ => Self::print_command_menu(),
            }
        }
    }

    fn print_mission_briefing(&self) {
        let g = &self.galaxy;
        let plural = if g.total_starbases != 1 { "S" } else { "" };
        println!(
            "YOU MUST DESTROY {} KLINGONS IN {} STARDATES WITH {} STARBASE{}",
            g.total_klingons, g.mission_duration as i32, g.total_starbases, plural,
        );
    }

    fn print_command_menu() {
        println!("   0 = SET COURSE");
        println!("   1 = SHORT RANGE SENSOR SCAN");
        println!("   2 = LONG RANGE SENSOR SCAN");
        println!("   3 = FIRE PHASERS");
        println!("   4 = FIRE PHOTON TORPEDOES");
        println!("   5 = SHIELD CONTROL");
        println!("   6 = DAMAGE CONTROL REPORT");
        println!("   7 = CALL ON LIBRARY COMPUTER");
    }

    fn read_line(prompt: &str) -> String {
        print!("{} ", prompt);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input
    }
}
