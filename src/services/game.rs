use crate::io::{InputReader, OutputWriter, TerminalIO};
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;
use crate::services::combat;
use crate::services::computer;
use crate::services::navigation;
use crate::services::scan;

pub struct Game {
    pub galaxy: Galaxy,
    io: TerminalIO,
    output: TerminalIO,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Game {
            galaxy: Galaxy::new(seed),
            io: TerminalIO,
            output: TerminalIO,
        }
    }

    pub fn run(&mut self) -> GameResult<()> {
        self.print_mission_briefing();
        scan::short_range_scan(&mut self.galaxy, &mut self.output)?;

        loop {
            let input = self.io.read_line("COMMAND")?;
            let input = input.trim();

            let result = match input {
                "0" => navigation::navigate(&mut self.galaxy, &mut self.io, &mut self.output),
                "1" => scan::short_range_scan(&mut self.galaxy, &mut self.output),
                "2" => scan::long_range_scan(&mut self.galaxy, &mut self.output),
                "3" => combat::fire_phasers(&mut self.galaxy, &mut self.io, &mut self.output),
                "4" => combat::fire_torpedoes(&mut self.galaxy, &mut self.io, &mut self.output),
                "5" => combat::shield_control(&mut self.galaxy, &mut self.io, &mut self.output),
                "6" => {
                    self.galaxy.enterprise().damage_report(&mut self.output);
                    Ok(())
                }
                "7" => computer::library_computer(&mut self.galaxy, &mut self.io, &mut self.output),
                "q" | "Q" => {
                    self.output.writeln("GOODBYE, CAPTAIN.");
                    break;
                }
                _ => {
                    Self::print_command_menu(&mut self.output);
                    Ok(())
                }
            };

            // Handle errors from commands - for now just print and continue
            if let Err(e) = result {
                self.output.writeln(&format!("Error: {}", e));
            }
        }
        Ok(())
    }

    fn print_mission_briefing(&mut self) {
        let g = &self.galaxy;
        let plural = if g.total_starbases() != 1 { "S" } else { "" };
        self.output.writeln(&format!(
            "YOU MUST DESTROY {} KLINGONS IN {} STARDATES WITH {} STARBASE{}",
            g.total_klingons(), g.mission_duration() as i32, g.total_starbases(), plural,
        ));
    }

    fn print_command_menu(output: &mut dyn OutputWriter) {
        output.writeln("   0 = SET COURSE");
        output.writeln("   1 = SHORT RANGE SENSOR SCAN");
        output.writeln("   2 = LONG RANGE SENSOR SCAN");
        output.writeln("   3 = FIRE PHASERS");
        output.writeln("   4 = FIRE PHOTON TORPEDOES");
        output.writeln("   5 = SHIELD CONTROL");
        output.writeln("   6 = DAMAGE CONTROL REPORT");
        output.writeln("   7 = CALL ON LIBRARY COMPUTER");
    }
}
