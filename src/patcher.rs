use crate::device::*;
use crate::error::UpdaterError;

#[derive(Debug, Clone, PartialEq)]
pub struct Patcher {
    device: Device,
    firmware: DeviceFirmware,
    custom_tty: Option<String>,
    dry_run: bool,
}

impl Patcher {
    pub fn new(device: Device, firmware: DeviceFirmware) -> Self {
        Self {
            device,
            firmware,
            custom_tty: None,
            dry_run: false
        }
    }

    pub fn target_tty(&mut self, tty: String) {
        self.custom_tty = Some(tty);
    }

    pub fn enable_dry_run(&mut self) {
        self.dry_run = true;
    }

    pub fn patch(self) -> Result<(), UpdaterError> {
        // TODO: Wrap avrdude and apply the hex
        Ok(())
    }
}
