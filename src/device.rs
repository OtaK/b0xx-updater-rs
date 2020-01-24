use crate::error::UpdaterError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceFirmware {
    pub version: String,
    pub date: toml::value::Datetime,
    pub active: bool,
    pub legal: bool,
    pub filename: String,
    pub filefmt: String,
}

impl std::fmt::Display for DeviceFirmware {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            r#"        - v{}
            - Active: {}
            - Legal: {}
            - Date released: {}
            - Hex information: {} / fmt {}
"#,
            self.version, self.active, self.legal, self.date, self.filename, self.filefmt
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub id: String,
    pub partno: String,
    pub progid: String,
    pub baudrate: u64,
    pub memtype: String,
    pub op: String,
    pub active: bool,
    pub firmwares: Vec<DeviceFirmware>,
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(
            f,
            r#"
    Device: {}
    Active: {}
    avrdude configuration:
        - partno: {}
        - progid: {}
        - baudrate: {}
        - memtype: {}
        - op: {}
    Firmwares:"#,
            self.name, self.active, self.partno, self.progid, self.baudrate, self.memtype, self.op
        )?;

        if self.firmwares.len() == 0 {
            writeln!(f, r#"        None"#)?;
        } else {
            for firmware in self.firmwares.iter() {
                writeln!(f, "{}", firmware)?;
            }
        }

        Ok(())
    }
}

impl Device {
    pub fn load_hardware_list() -> Result<Vec<Self>, UpdaterError> {
        let map: std::collections::HashMap<String, Device> =
            toml::from_slice(&std::fs::read("assets/hardware.toml")?)?;

        Ok(map.into_iter().map(|(_, v)| v).collect())
    }

    #[allow(dead_code)]
    pub fn list_visible_firmwares(&self) -> impl std::iter::Iterator<Item = &DeviceFirmware> {
        self.firmwares.iter().filter(|f| f.active)
    }
}
