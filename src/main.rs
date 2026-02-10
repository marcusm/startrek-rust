mod cli;
mod game_engine;
mod models;
mod services;
mod io;
mod ui;

use std::io::{self as stdio, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::args::parse();

    // Centered title
    print_centered("STAR TREK", 80);
    println!();

    // Instructions prompt (only if no seed provided via CLI)
    if args.seed.is_none() {
        print!("ENTER 1 OR 2 FOR INSTRUCTIONS (ENTER 2 TO PAGE) ");
        stdio::stdout().flush()?;
        let mut input = String::new();
        stdio::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => show_instructions(false),
            "2" => show_instructions(true),
            _ => {} // Skip instructions
        }
    }

    // Seed prompt (only if not provided via CLI)
    let seed: u64 = if let Some(s) = args.seed {
        s
    } else {
        print!("ENTER SEED NUMBER ");
        stdio::stdout().flush()?;
        let mut input = String::new();
        stdio::stdin().read_line(&mut input)?;
        input.trim().parse().unwrap_or(0)
    };

    println!("INITIALIZING...");
    let mut game = services::game::Game::new(seed);
    game.run()?;
    Ok(())
}

/// Print text centered within a given width.
fn print_centered(text: &str, width: usize) {
    let padding = (width.saturating_sub(text.len())) / 2;
    println!("{:>width$}", text, width = padding + text.len());
}

/// Display game instructions, optionally paged.
fn show_instructions(paged: bool) {
    let instructions = vec![
        "INSTRUCTIONS FOR STAR TREK",
        "",
        "YOU ARE CAPTAIN OF THE STARSHIP ENTERPRISE. YOUR MISSION IS TO",
        "DESTROY ALL KLINGON BATTLE CRUISERS IN THE GALAXY BEFORE TIME",
        "RUNS OUT.",
        "",
        "THE GALAXY IS DIVIDED INTO AN 8X8 GRID OF QUADRANTS.",
        "EACH QUADRANT IS FURTHER DIVIDED INTO AN 8X8 GRID OF SECTORS.",
        "",
        "COMMANDS:",
        "  0 = SET COURSE           Navigate to a new location",
        "  1 = SHORT RANGE SCAN     View current quadrant",
        "  2 = LONG RANGE SCAN      View surrounding quadrants",
        "  3 = FIRE PHASERS         Attack with phasers",
        "  4 = FIRE TORPEDOES       Attack with photon torpedoes",
        "  5 = SHIELD CONTROL       Transfer energy to/from shields",
        "  6 = DAMAGE REPORT        View status of ship systems",
        "  7 = LIBRARY COMPUTER     Access computer functions",
        "",
        "SHIP SYSTEMS:",
        "  Each system can be damaged during combat or navigation.",
        "  Damaged systems are repaired slowly during warp travel.",
        "",
        "DOCKING:",
        "  Move adjacent to a starbase to dock automatically.",
        "  Docking restores energy, shields, and torpedoes.",
        "",
        "STRATEGY TIPS:",
        "  - Keep shields up when Klingons are present",
        "  - Dock at starbases to repair and resupply",
        "  - Use long range sensors to plan your route",
        "  - Watch your energy and time remaining",
        "",
        "GOOD LUCK, CAPTAIN!",
        "",
    ];

    if paged {
        // Display 20 lines at a time
        for (i, line) in instructions.iter().enumerate() {
            println!("{}", line);
            if (i + 1) % 20 == 0 && i + 1 < instructions.len() {
                print!("-- PRESS ENTER TO CONTINUE -- ");
                stdio::stdout().flush().unwrap();
                let mut input = String::new();
                stdio::stdin().read_line(&mut input).unwrap();
            }
        }
    } else {
        for line in instructions {
            println!("{}", line);
        }
    }
    println!();
}
