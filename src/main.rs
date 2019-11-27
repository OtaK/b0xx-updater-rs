#[macro_use]
extern crate log;

mod device;
mod error;
mod patcher;

use clap::{clap_app, crate_authors, crate_description, crate_version};

fn main() -> Result<(), error::UpdaterError> {
    if let Ok(env) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", format!("b0xx_viewer=info,{}", env));
    } else {
        std::env::set_var("RUST_LOG", "b0xx_viewer=info");
    }

    pretty_env_logger::init();

    debug!("Loading hardware list...");
    let hardware_list = device::Device::load_hardware_list()?;
    debug!("Loaded hardware list:\n{:#?}", hardware_list);

    let matches = clap_app!(b0xx_updater =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg list: -l --list "Lists available patches.")
        (@arg device: -d --device +takes_value "Specify which device you want to patch.")
        (@arg patch_version: -p --patch +takes_value "Specify which firmware version you want to use.")
        (@arg custom_tty: --tty +takes_value "Specify a custom TTY/COM port on which to perform the patch.")
        (@arg dry_run: --dry_run "Lists actions that will be performed without actually performing them.")
    ).get_matches();

    if matches.is_present("list") {
        for device in hardware_list {
            print!("{}", device);
        }

        return Ok(())
    }

    let dry_run = matches.is_present("dry_run");

    if let Some(device_id) = matches.value_of("device").take() {
        if let Some(patch_version) = matches.value_of("patch_version").take() {
            if let Some(device) = hardware_list.into_iter().find(|d| d.id == device_id) {
                if let Some(patch) = device.firmwares.iter().find(|p| p.version == patch_version) {
                    let firmware = patch.clone();
                    let mut patcher = patcher::Patcher::new(device, firmware);
                    if let Some(tty) = matches.value_of("custom_tty").take() {
                        patcher.target_tty(tty.into());
                    }
                    if dry_run {
                        patcher.enable_dry_run();
                    }
                    patcher.patch()?;
                    info!("Patch successfully applied!");
                }
            }
        } else {
            error!("Device has been selected but patch version has not been!");
        }
    } else {
        error!("Device has not been selected!");
    }


    Ok(())
}
