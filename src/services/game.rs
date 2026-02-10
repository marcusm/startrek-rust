use crate::game_engine::{GameEngine, GameState, DefeatReason};
use crate::io::{InputReader, OutputWriter, TerminalIO};
use crate::models::errors::GameResult;
use crate::services::combat;
use crate::services::computer;
use crate::services::navigation;
use crate::services::scan;
use crate::ui::presenters::{EnterprisePresenter, CombatPresenter};

pub struct Game {
    game_engine: GameEngine,
    io: TerminalIO,
    output: TerminalIO,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Game {
            game_engine: GameEngine::new(seed),
            io: TerminalIO,
            output: TerminalIO,
        }
    }

    pub fn run(&mut self) -> GameResult<()> {
        self.print_mission_briefing();
        scan::short_range_scan(self.game_engine.galaxy_mut(), &mut self.output)?;

        loop {
            let input = self.io.read_line("COMMAND")?;
            let input = input.trim();

            let result = match input {
                "0" => navigation::navigate(self.game_engine.galaxy_mut(), &mut self.io, &mut self.output),
                "1" => scan::short_range_scan(self.game_engine.galaxy_mut(), &mut self.output),
                "2" => scan::long_range_scan(self.game_engine.galaxy_mut(), &mut self.output),
                "3" => combat::fire_phasers(self.game_engine.galaxy_mut(), &mut self.io, &mut self.output),
                "4" => combat::fire_torpedoes(self.game_engine.galaxy_mut(), &mut self.io, &mut self.output),
                "5" => combat::shield_control(self.game_engine.galaxy_mut(), &mut self.io, &mut self.output),
                "6" => {
                    EnterprisePresenter::show_damage_report(self.game_engine.galaxy().enterprise(), &mut self.output);
                    Ok(())
                }
                "7" => computer::library_computer(self.game_engine.galaxy_mut(), &mut self.io, &mut self.output),
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

            // Check for game over after each command
            if let Some(state) = self.game_engine.check_game_over() {
                match state {
                    GameState::Victory { rating } => {
                        CombatPresenter::show_victory(rating, &mut self.output);
                        break;
                    }
                    GameState::Defeat { reason } => {
                        let message = match reason {
                            DefeatReason::ShipDestroyed => "SHIP DESTROYED",
                            DefeatReason::TimeExpired => "TIME EXPIRED",
                            DefeatReason::DeadInSpace => "DEAD IN SPACE",
                        };
                        CombatPresenter::show_defeat(message, &mut self.output);
                        break;
                    }
                    GameState::Playing => {} // Continue playing
                }
            }
        }
        Ok(())
    }

    fn print_mission_briefing(&mut self) {
        let g = self.game_engine.galaxy();
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
