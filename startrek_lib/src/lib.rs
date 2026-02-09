/// Represents a starship in the game
#[derive(Debug, Clone)]
pub struct Starship {
    pub name: String,
    pub energy: u32,
    pub shields: u32,
}

impl Starship {
    /// Creates a new starship with default values
    pub fn new(name: String) -> Self {
        Self {
            name,
            energy: 100,
            shields: 100,
        }
    }

    /// Fires torpedoes, consuming energy
    pub fn fire_torpedoes(&mut self) -> Result<(), String> {
        if self.energy >= 10 {
            self.energy -= 10;
            Ok(())
        } else {
            Err("Not enough energy to fire torpedoes".to_string())
        }
    }

    /// Recharges shields using energy
    pub fn recharge_shields(&mut self, amount: u32) -> Result<(), String> {
        if self.energy >= amount {
            self.energy -= amount;
            self.shields = (self.shields + amount).min(100);
            Ok(())
        } else {
            Err("Not enough energy to recharge shields".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_starship() {
        let ship = Starship::new("Enterprise".to_string());
        assert_eq!(ship.name, "Enterprise");
        assert_eq!(ship.energy, 100);
        assert_eq!(ship.shields, 100);
    }

    #[test]
    fn test_fire_torpedoes() {
        let mut ship = Starship::new("Enterprise".to_string());
        assert!(ship.fire_torpedoes().is_ok());
        assert_eq!(ship.energy, 90);
    }

    #[test]
    fn test_fire_torpedoes_low_energy() {
        let mut ship = Starship::new("Enterprise".to_string());
        ship.energy = 5;
        assert!(ship.fire_torpedoes().is_err());
    }

    #[test]
    fn test_recharge_shields() {
        let mut ship = Starship::new("Enterprise".to_string());
        ship.shields = 50;
        assert!(ship.recharge_shields(20).is_ok());
        assert_eq!(ship.shields, 70);
        assert_eq!(ship.energy, 80);
    }
}
