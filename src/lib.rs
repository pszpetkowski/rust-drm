mod core;

pub mod bus;
pub mod device;
pub mod node;

pub use bus::DrmBus;
pub use device::{BusInfo, DeviceInfo, DrmDevice, PCIBusInfo, PCIDeviceInfo};
pub use node::{DrmNode, DrmNodeType};

use crate::core::Result;
use std::fs::File;
use std::io::Read;

pub fn get_uevent_data_by_key(pci_path: std::path::PathBuf, entry_key: &str) -> String {
    let uevent_path = pci_path.join("uevent");
    let mut uevent_text: String = String::new();
    File::open(uevent_path)
        .unwrap()
        .read_to_string(&mut uevent_text)
        .unwrap();

    uevent_text
        .split("\n")
        .filter(|entry| entry.starts_with(entry_key))
        .map(|entry| entry.split("=").last().unwrap())
        .collect()
}

pub fn process_device(
    device_name: &str,
    expected_subsystem_type: Option<DrmBus>,
) -> Result<DrmDevice> {
    let drm_node = DrmNode::from_device_name(device_name)?;
    if !drm_node.device_dir_exists() {
        return Err("Device dir for a given DRM Node does not exist")?;
    }

    let subsystem_type = DrmBus::get_subsystem_type(&drm_node)?;
    if let Some(expected) = expected_subsystem_type {
        if expected != subsystem_type {
            return Err("Expected subsystem type does not match with node")?;
        }
    }

    let node_type = DrmNodeType::from_minor_name(device_name)?;

    match subsystem_type {
        DrmBus::PCI => {
            let pci_path = drm_node.get_device_path();
            let pci_slot_name = get_uevent_data_by_key(pci_path, "PCI_SLOT_NAME");
            let bus_info = BusInfo::Pci(PCIBusInfo::new(&pci_slot_name));
            let device_info = DeviceInfo::Pci(PCIDeviceInfo::new(&drm_node));
            Ok(DrmDevice::new(
                node_type,
                subsystem_type,
                bus_info,
                device_info,
            ))
        }
        _ => Err("Unsupported DRM subsystem type")?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_device_happy_path() {
        let device = process_device("renderD128", Some(DrmBus::PCI)).unwrap();
        dbg!(device);
    }
}
