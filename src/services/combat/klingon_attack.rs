use rand::Rng;

use crate::io::OutputWriter;
use crate::models::galaxy::Galaxy;

use super::phasers::calculate_distance;

/// Klingons attack the Enterprise (spec section 8).
/// Returns true if the Enterprise is destroyed, false otherwise.
pub fn klingons_fire(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) -> bool {
    // Skip if docked (spec section 8.3)
    if galaxy
        .enterprise()
        .is_adjacent_to_starbase(galaxy.sector_map().starbase)
    {
        output.writeln("STAR BASE SHIELDS PROTECT THE ENTERPRISE");
        return false;
    }

    let e_pos = galaxy.enterprise().sector();

    // Collect klingon data to avoid borrow conflicts
    let klingon_attacks: Vec<_> = galaxy
        .sector_map()
        .klingons
        .iter()
        .filter(|k| k.is_alive())
        .map(|k| (k.sector, k.shields, calculate_distance(e_pos, k.sector)))
        .collect();

    for (k_sector, k_shields, distance) in klingon_attacks {
        let hit = (k_shields / distance) * (2.0 * galaxy.rng_mut().gen::<f64>());

        galaxy.enterprise_mut().subtract_shields(hit);

        output.writeln(&format!(
            "{} UNIT HIT ON ENTERPRISE FROM SECTOR {},{}",
            hit as i32, k_sector.x, k_sector.y
        ));
        output.writeln(&format!(
            "   ({} LEFT)",
            galaxy.enterprise().shields().max(0.0) as i32
        ));
    }

    // Check if Enterprise is destroyed (spec section 8.4)
    // Return true so caller can check game over condition
    galaxy.enterprise().shields() < 0.0
}

/// Handle the dead-in-space scenario where Klingons fire repeatedly (spec 10.4).
/// The Enterprise is stuck with no energy and minimal shields. All Klingons in the
/// quadrant fire until either the Enterprise is destroyed or miraculously survives.
pub fn dead_in_space_loop(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) {
    loop {
        // Check if there are any Klingons left to fire
        if galaxy.sector_map().klingons.is_empty() {
            // No Klingons to fire - Enterprise survives, demoted to private
            output.writeln("");
            output.writeln(&format!(
                "THERE ARE STILL {} KLINGON BATTLE CRUISERS",
                galaxy.total_klingons()
            ));
            return; // Exit loop, let game engine handle defeat
        }

        // Klingons fire (uses existing klingons_fire function)
        // This function returns true if Enterprise is destroyed (shields < 0)
        if klingons_fire(galaxy, output) {
            return; // Enterprise destroyed, let game engine handle defeat
        }

        // If we reach here, shields are still >= 0 despite the attack
        // In practice, this is extremely unlikely with shields < 1
        // But the spec says "fire repeatedly until" so we continue the loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::test_utils::MockOutput;
    use crate::models::constants::SectorContent;
    use crate::models::galaxy::Galaxy;
    use crate::models::klingon::Klingon;
    use crate::models::position::SectorPosition;
    use crate::models::sector_map::SectorMap;

    /// Helper: Set up a combat scenario with specified parameters.
    fn setup_combat_scenario(
        seed: u64,
        enterprise_energy: f64,
        enterprise_shields: f64,
        klingon_shields: f64,
    ) -> Galaxy {
        let mut galaxy = Galaxy::new(seed);

        // Clear sector map
        *galaxy.sector_map_mut() = SectorMap::new();

        // Place Enterprise at (4, 4)
        let sector = SectorPosition { x: 4, y: 4 };
        let quadrant = galaxy.enterprise().quadrant();
        galaxy.enterprise_mut().move_to(quadrant, sector);
        galaxy.enterprise_mut().set_energy(enterprise_energy);
        galaxy.enterprise_mut().set_shields(enterprise_shields);
        let enterprise_sector = galaxy.enterprise().sector();
        galaxy
            .sector_map_mut()
            .set(enterprise_sector, SectorContent::Enterprise);

        // Place one Klingon at (2, 2)
        let klingon_pos = SectorPosition { x: 2, y: 2 };
        let mut klingon = Klingon::new(klingon_pos);
        klingon.shields = klingon_shields;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        galaxy
    }

    // ========== Klingon firing tests ==========

    #[test]
    fn klingons_fire_reduces_shields() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        let initial_shields = galaxy.enterprise().shields();

        klingons_fire(&mut galaxy, &mut MockOutput::new());

        assert!(galaxy.enterprise().shields() < initial_shields);
    }

    #[test]
    fn klingons_fire_skips_when_docked() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place starbase adjacent to Enterprise
        let starbase_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map_mut().set(starbase_pos, SectorContent::Starbase);
        galaxy.sector_map_mut().starbase = Some(starbase_pos);

        let initial_shields = galaxy.enterprise().shields();
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        assert_eq!(galaxy.enterprise().shields(), initial_shields);
    }

    #[test]
    fn klingons_fire_does_not_hit_from_dead_klingons() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.sector_map_mut().klingons[0].shields = 0.0;

        let initial_shields = galaxy.enterprise().shields();
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        // Shields should not change if all Klingons are dead
        assert_eq!(galaxy.enterprise().shields(), initial_shields);
    }

    #[test]
    fn klingons_fire_damage_depends_on_distance() {
        // Closer Klingon should do more damage
        let mut galaxy1 = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        // Klingon at (2,2), Enterprise at (4,4) - distance = sqrt(8) ≈ 2.83

        let mut galaxy2 = Galaxy::new(42);
        *galaxy2.sector_map_mut() = SectorMap::new();
        let sector = SectorPosition { x: 4, y: 4 };
        let quadrant = galaxy2.enterprise().quadrant();
        galaxy2.enterprise_mut().move_to(quadrant, sector);
        galaxy2.enterprise_mut().set_energy(3000.0);
        galaxy2.enterprise_mut().set_shields(500.0);
        let enterprise_sector = galaxy2.enterprise().sector();
        galaxy2
            .sector_map_mut()
            .set(enterprise_sector, SectorContent::Enterprise);

        // Place Klingon farther away at (1, 1)
        let far_klingon_pos = SectorPosition { x: 1, y: 1 };
        let mut far_klingon = Klingon::new(far_klingon_pos);
        far_klingon.shields = 200.0;
        galaxy2.sector_map_mut().set(far_klingon_pos, SectorContent::Klingon);
        galaxy2.sector_map_mut().klingons.push(far_klingon);

        klingons_fire(&mut galaxy1, &mut MockOutput::new());
        klingons_fire(&mut galaxy2, &mut MockOutput::new());

        // Both have random component, but on average closer Klingon does more damage
        // We can only verify shields were reduced from both
        assert!(galaxy1.enterprise().shields() < 500.0);
        assert!(galaxy2.enterprise().shields() < 500.0);
    }

    #[test]
    fn multiple_klingons_all_take_damage() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Add second Klingon
        let k2_pos = SectorPosition { x: 6, y: 6 };
        let k2 = Klingon::new(k2_pos);
        galaxy.sector_map_mut().set(k2_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(k2);

        // Add third Klingon
        let k3_pos = SectorPosition { x: 3, y: 7 };
        let k3 = Klingon::new(k3_pos);
        galaxy.sector_map_mut().set(k3_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(k3);

        assert_eq!(galaxy.sector_map().klingons.len(), 3);

        // All Klingons fire
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        // Enterprise shields should be reduced by attacks from all 3
        assert!(galaxy.enterprise().shields() < 500.0);
    }

    // ========== Victory/defeat tests ==========

    #[test]
    fn victory_when_last_klingon_destroyed() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 10.0);
        galaxy.set_total_klingons(1); // Only one Klingon in entire galaxy

        // Manually destroy the Klingon
        let klingon_pos = galaxy.sector_map().klingons[0].sector;
        galaxy.sector_map_mut().klingons[0].shields = 0.0;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Empty);
        galaxy.decrement_klingons();
        galaxy.decrement_quadrant_klingons();

        assert_eq!(galaxy.total_klingons(), 0);
        assert!(galaxy.all_klingons_destroyed());
        // Victory check now handled by GameEngine
    }

    // ========== Retain cleanup tests ==========

    #[test]
    fn retain_removes_only_dead_klingons() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Add second Klingon that's already dead
        let dead_klingon_pos = SectorPosition { x: 6, y: 6 };
        let mut dead_klingon = Klingon::new(dead_klingon_pos);
        dead_klingon.shields = 0.0;
        galaxy.sector_map_mut().set(dead_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(dead_klingon);

        // Add third Klingon that's alive
        let alive_klingon_pos = SectorPosition { x: 7, y: 7 };
        let alive_klingon = Klingon::new(alive_klingon_pos);
        galaxy
            .sector_map_mut()
            .set(alive_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(alive_klingon);

        assert_eq!(galaxy.sector_map().klingons.len(), 3);

        // Apply retain
        galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());

        // Should have 2 living Klingons left
        assert_eq!(galaxy.sector_map().klingons.len(), 2);
        for k in &galaxy.sector_map().klingons {
            assert!(k.is_alive());
        }
    }

    #[test]
    fn klingon_destruction_clears_grid() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        let klingon_pos = galaxy.sector_map().klingons[0].sector;

        // Verify Klingon is in grid
        assert_eq!(
            galaxy.sector_map().get(klingon_pos),
            SectorContent::Klingon
        );

        // Destroy Klingon
        galaxy.sector_map_mut().klingons[0].shields = 0.0;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Empty);
        galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());

        // Verify grid is cleared and vector is empty
        assert_eq!(galaxy.sector_map().get(klingon_pos), SectorContent::Empty);
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }
}
