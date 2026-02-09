mod cli;
mod models;
mod services;

fn main() {
    println!("*** STAR TREK ***");
    println!();

    let game = services::game::Game::new();
    game.run();
}
