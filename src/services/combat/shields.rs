use crate::io::{InputReader, OutputWriter};
use crate::models::constants::Device;
use crate::models::enterprise::ShieldControlError;
use crate::models::errors::{GameError, GameResult};
use crate::models::galaxy::Galaxy;

/// Transfers energy between shields and main power reserves (Command 5)
///
/// Allows the player to allocate energy between shields and main power.
/// The total energy (shields + main power) remains constant during transfer.
/// Positive values transfer energy to shields, negative values transfer from shields.
///
/// # Arguments
///
/// * `galaxy` - The game galaxy state
/// * `io` - Input reader for getting transfer amount
/// * `output` - Output writer for displaying available energy
///
/// # Returns
///
/// * `Ok(())` on successful transfer or cancellation
/// * `Err(GameError::InsufficientResources)` if insufficient energy available
/// * `Err` for other I/O failures
///
/// # Specification
///
/// See spec section 6.5 for full details on shield control mechanics.
pub fn shield_control(
    galaxy: &mut Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    // Check if shield control is damaged (spec section 6.5)
    if galaxy.enterprise().is_damaged(Device::ShieldControl) {
        output.writeln("SHIELD CONTROL IS NON-OPERATIONAL");
        return Ok(());
    }

    // Display available energy (energy + shields)
    let total_energy = galaxy.enterprise().energy() + galaxy.enterprise().shields();
    output.writeln(&format!("ENERGY AVAILABLE = {}", total_energy as i32));

    // Prompt for input
    let input = io.read_line("NUMBER OF UNITS TO SHIELDS")?;
    let units: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(()), // Invalid parse, return to command prompt
    };

    // If input ≤ 0, return to command prompt (spec section 6.5)
    if units <= 0.0 {
        return Ok(());
    }

    // Attempt to transfer energy
    match galaxy.enterprise_mut().shield_control(units) {
        Ok(()) => {
            // Success - energy transferred, return to command prompt
        }
        Err(ShieldControlError::InsufficientEnergy) => {
            // Return error instead of recursion - caller will handle retry
            return Err(GameError::InsufficientResources {
                required: units,
                available: total_energy,
            });
        }
        Err(ShieldControlError::InvalidInput) => {
            // Return to command prompt
        }
        Err(ShieldControlError::SystemDamaged) => {
            // Should never happen - we checked above
        }
    }
    Ok(())
}
