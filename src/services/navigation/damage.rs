use crate::io::OutputWriter;
use crate::models::constants::Device;
use crate::models::galaxy::Galaxy;

/// Automatic device repair on navigation moves (spec section 5.2).
/// Each damaged device (value < 0) is incremented by 1.
pub fn auto_repair_devices(galaxy: &mut Galaxy) {
    for device in Device::ALL.iter() {
        if galaxy.enterprise().is_damaged(*device) {
            galaxy.enterprise_mut().repair_device(*device, 1.0);
        }
    }
}

/// Random damage/repair events on navigation moves (spec section 5.3).
/// 20% chance of event affecting a random device.
/// FIXED: Now uses galaxy.rng instead of thread_rng() for determinism
pub fn random_damage_event(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) {
    use rand::Rng;

    // 20% chance of event - FIXED: using galaxy.rng for determinism!
    if galaxy.rng_mut().gen::<f64>() > 0.2 {
        return;
    }

    // Select random device (0-7 index)
    let device_index = (galaxy.rng_mut().gen::<f64>() * 8.0).floor() as usize;

    // Determine severity (1-5)
    let severity = (galaxy.rng_mut().gen::<f64>() * 5.0).floor() + 1.0;

    // 50% chance of damage vs repair
    let is_repair = galaxy.rng_mut().gen::<f64>() >= 0.5;

    let device = Device::ALL[device_index];

    output.writeln("");
    if is_repair {
        galaxy.enterprise_mut().repair_device(device, severity);
        output.writeln(&format!(
            "DAMAGE CONTROL REPORT: {} STATE OF REPAIR IMPROVED",
            device.name()
        ));
    } else {
        galaxy.enterprise_mut().damage_device(device, severity);
        output.writeln(&format!("DAMAGE CONTROL REPORT: {} DAMAGED", device.name()));
    }
    output.writeln("");
}
