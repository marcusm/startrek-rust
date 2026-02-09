mod cli;
mod models;
mod services;

fn main() {
    let args = cli::args::parse();

    println!("*** STAR TREK ***");
    println!();

    let seed: u64 = args.seed.unwrap_or(0);

    println!("INITIALIZING...");
    let mut game = services::game::Game::new(seed);
    game.run();
}
