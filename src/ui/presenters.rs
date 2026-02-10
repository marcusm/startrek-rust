use crate::io::OutputWriter;
use crate::models::constants::Device;
use crate::models::enterprise::Enterprise;
use crate::models::position::SectorPosition;

pub struct EnterprisePresenter;

impl EnterprisePresenter {
    pub fn show_damage_report(enterprise: &Enterprise, output: &mut dyn OutputWriter) {
        if enterprise.is_damaged(Device::DamageControl) {
            output.writeln("DAMAGE CONTROL REPORT IS NOT AVAILABLE");
            return;
        }

        output.writeln(&format!("{:<14}{}", "DEVICE", "STATE OF REPAIR"));
        for device in Device::ALL.iter() {
            let state = enterprise.devices()[*device as usize] as i32;
            output.writeln(&format!("{:<14}{}", device.name(), state));
        }
    }
}

pub struct CombatPresenter;

impl CombatPresenter {
    pub fn show_klingon_hit(hit: f64, pos: SectorPosition, remaining: f64, output: &mut dyn OutputWriter) {
        output.writeln(&format!(
            "{} UNIT HIT ON KLINGON AT SECTOR {},{}",
            hit as i32, pos.x, pos.y
        ));
        output.writeln(&format!("   ({} LEFT)", remaining.max(0.0) as i32));
    }

    pub fn show_klingon_destroyed(output: &mut dyn OutputWriter) {
        output.writeln("*** KLINGON DESTROYED ***");
    }

    pub fn show_victory(rating: i32, output: &mut dyn OutputWriter) {
        output.writeln("");
        output.writeln("THE LAST KLINGON BATTLE CRUISER IN THE GALAXY HAS BEEN DESTROYED");
        output.writeln("THE FEDERATION HAS BEEN SAVED !!!");
        output.writeln("");
        output.writeln(&format!("YOUR EFFICIENCY RATING = {}", rating));
    }

    pub fn show_defeat(reason: &str, output: &mut dyn OutputWriter) {
        output.writeln("");
        output.writeln(&format!("*** {}", reason));
        output.writeln("THE FEDERATION WILL BE CONQUERED");
        output.writeln("");
    }
}
