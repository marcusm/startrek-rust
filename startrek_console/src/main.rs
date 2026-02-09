use startrek_lib::Starship;

fn main() {
    println!("=== Star Trek Console ===");
    println!();

    // Create a new starship
    let mut ship = Starship::new("USS Enterprise".to_string());
    println!("Created starship: {}", ship.name);
    println!("Energy: {}, Shields: {}", ship.energy, ship.shields);
    println!();

    // Fire torpedoes
    println!("Firing torpedoes...");
    match ship.fire_torpedoes() {
        Ok(_) => println!("Torpedoes fired successfully!"),
        Err(e) => println!("Error: {}", e),
    }
    println!("Energy: {}, Shields: {}", ship.energy, ship.shields);
    println!();

    // Reduce shields to demonstrate recharging
    ship.shields = 60;
    println!("Shields damaged! Shields: {}", ship.shields);
    
    // Recharge shields
    println!("Recharging shields...");
    match ship.recharge_shields(20) {
        Ok(_) => println!("Shields recharged!"),
        Err(e) => println!("Error: {}", e),
    }
    println!("Energy: {}, Shields: {}", ship.energy, ship.shields);
    println!();

    println!("Mission complete!");
}
