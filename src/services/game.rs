pub struct Game;

impl Game {
    pub fn new() -> Self {
        Game
    }

    pub fn run(&self) {
        println!("Mission briefing: Destroy all Klingon warships.");
        println!("The Federation is counting on you, Captain!");
        println!();
        println!("Game over. Goodbye, Captain.");
    }
}
