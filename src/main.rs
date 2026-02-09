mod cli;
mod models;
mod services;

fn main() {
    println!("*** STAR TREK ***");
    println!();

    // Future: prompt for seed ("ENTER SEED NUMBER")
    let seed: u64 = 0;

    println!("INITIALIZING...");
    let mut game = services::game::Game::new(seed);
    game.run();
}
