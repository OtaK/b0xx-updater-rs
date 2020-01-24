use crate::device::*;
use crate::error::UpdaterError;

#[derive(Debug, Clone, PartialEq)]
pub struct Patcher {
    device: Device,
    firmware: DeviceFirmware,
    custom_tty: Option<String>,
    dfu_tty: Option<String>,
    old_ports: Option<Vec<String>>,
    dry_run: bool,
}

impl Patcher {
    pub fn detect_avrdude() -> bool {
        if cfg!(windows) {
            if !std::path::Path::new("vendor/avrdude/windows/avrdude.exe").exists() {
                error!(
                    "avrdude is missing in the vendor folder! Did you unzip the tarball correctly?"
                );
                return false;
            }

            debug!("[WINDOWS] avrdude executable found");
        } else {
            if let Err(e) = std::process::Command::new("avrdude").output() {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        if cfg!(target_os = "macos") {
                            error!("avrdude has not been found on your system, please install it with Brew using `brew install avrdude` and try again.");
                        } else if cfg!(target_os = "linux") {
                            error!("avrdude has not been found on your system, please install it using your system's package manager (eg. `sudo apt-get install avrdude`) and try again.");
                        } else {
                            error!("avrdude has not been found on your system, and your OS variant is unsupported, install avrdude in your $PATH by whatever means you have and try again.");
                        }
                    }
                    _ => {}
                }

                return false;
            }

            debug!("[UNIX] avrdude executable found");
        }

        true
    }

    pub fn new(device: Device, firmware: DeviceFirmware) -> Self {
        Self {
            device,
            firmware,
            custom_tty: None,
            dfu_tty: None,
            old_ports: None,
            dry_run: false,
        }
    }

    pub fn target_tty(&mut self, tty: String) {
        self.custom_tty = Some(tty);
    }

    pub fn enable_dry_run(&mut self) {
        self.dry_run = true;
    }

    fn detect_b0xx(&mut self) -> Result<(), UpdaterError> {
        if self.custom_tty.is_some() {
            return Ok(());
        }

        let ports = serialport::available_ports()?;
        debug!("Ports: {:#?}", ports);
        self.old_ports = Some(ports.iter().map(|p| p.port_name.clone()).collect());

        let b0xx_port = ports
            .into_iter()
            .find(move |port| {
                debug!("Found TTY port: {:?}", port);
                if let serialport::SerialPortType::UsbPort(portinfo) = &port.port_type {
                    if portinfo.vid == 9025 && portinfo.pid == 32822 {
                        return true;
                    }

                    if let Some(product) = &portinfo.product {
                        if product == "Arduino_Leonardo" {
                            return true;
                        }
                    }
                }

                false
            })
            .ok_or_else(|| UpdaterError::B0xxNotFound)?;

        debug!("Found B0XX on port {}", b0xx_port.port_name);
        self.custom_tty = Some(b0xx_port.port_name);
        Ok(())
    }

    pub fn patch(mut self) -> Result<(), UpdaterError> {
        // Force b0xx detection in case it hasn't been performed yet
        let _ = self.detect_b0xx()?;

        let tty = self.custom_tty.as_ref().unwrap();

        // Enable DFU
        let dfu_serial_settings = serialport::SerialPortSettings {
            baud_rate: 1200,
            ..Default::default()
        };

        debug!(
            "Opening DFU mode on port {} with {:#?}",
            tty, dfu_serial_settings
        );
        let dfu_activation_port = Some(serialport::open_with_settings(tty, &dfu_serial_settings)?);
        debug!("DFU mode started, sleeping 1.5 seconds for new port to appear...");
        std::thread::sleep(std::time::Duration::from_millis(3000));
        debug!("Scanning new ports...");
        let ports = serialport::available_ports()?;
        debug!("New port list: {:#?}", ports);

        let old_ports = self.old_ports.take().unwrap_or_default();

        let dfu_tty = if let Some(tty) = ports
            .into_iter()
            .find(move |port| !old_ports.contains(&port.port_name))
        {
            tty
        } else {
            error!("DFU TTY not found!");
            return Err(UpdaterError::DfuTtyNotFound);
        };

        debug!("Found DFU TTY: {:#?}", dfu_tty);

        // Apply patch
        let mut cmd = std::process::Command::new(if cfg!(windows) {
            "./vendor/avrdude/windows/avrdude.exe"
        } else {
            "avrdude"
        });
        cmd.args(&[
            "-C",
            "./vendor/avrdude/avrdude.conf",
            "-p",
            &self.device.partno,
            "-c",
            &self.device.progid,
            "-P",
            &dfu_tty.port_name,
            "-b",
            &format!("{}", self.device.baudrate),
            "-D",
            "-U",
            &format!(
                r#"{}:{}:./assets/hex/{}:{}"#,
                self.device.memtype, self.device.op, self.firmware.filename, self.firmware.filefmt,
            ),
        ]);

        if self.dry_run {
            info!("About to perform command: {:?}", cmd);
            info!("Since dry run is enabled and your B0XX has been put in DFU mode, please unplug and plug it back.");
        } else {
            let status = cmd.spawn()?.wait_with_output()?;

            debug!("avrdude status: {:#?}", status);
        }

        drop(dfu_activation_port);

        Ok(())
    }
}
