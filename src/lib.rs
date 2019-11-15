mod core;

pub mod bus;
pub mod node;

pub use bus::DrmBus;
pub use node::{DrmNode, DrmNodeType};

use crate::core::Result;
use regex::Regex;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct PCIBusInfo {
    domain: u16,
    bus: u8,
    dev: u8,
    func: u8,
}

impl PCIBusInfo {
    fn new(pci_slot_name: &str) -> PCIBusInfo {
        let pci_info_re =
            Regex::new(r"([0-9a-fA-F]{4}):([0-9a-fA-F]{2}):([0-9a-fA-F]{2}).(\d)").unwrap();

        let caps = pci_info_re.captures(pci_slot_name).unwrap();
        let domain: u16 = u16::from_str_radix(caps.get(1).unwrap().as_str(), 16).unwrap();
        let bus: u8 = u8::from_str_radix(caps.get(2).unwrap().as_str(), 16).unwrap();
        let dev: u8 = u8::from_str_radix(caps.get(3).unwrap().as_str(), 16).unwrap();
        let func: u8 = caps.get(4).unwrap().as_str().parse().unwrap();

        PCIBusInfo {
            domain,
            bus,
            dev,
            func,
        }
    }
}

#[derive(Debug)]
pub enum BusInfo {
    Pci(PCIBusInfo),
    Usb,
    Platform,
    Host1x,
}

#[derive(Debug)]
pub struct PCIDeviceInfo {
    vendor_id: u16,
    device_id: u16,
    revision_id: u8,
    subvendor_id: u16,
    subdevice_id: u16,
}

impl PCIDeviceInfo {
    fn new(drm_node: &DrmNode) -> PCIDeviceInfo {
        let config_path = drm_node.get_config_path();
        let mut buffer = [0; 64];
        File::open(config_path)
            .unwrap()
            .read(&mut buffer[..])
            .unwrap();

        let vendor_id = buffer[0] as u16 | ((buffer[1] as u16) << 8);
        let device_id = buffer[2] as u16 | ((buffer[3] as u16) << 8);
        let revision_id = buffer[8];
        let subvendor_id = buffer[44] as u16 | ((buffer[45] as u16) << 8);
        let subdevice_id = buffer[46] as u16 | ((buffer[47] as u16) << 8);

        PCIDeviceInfo {
            vendor_id,
            device_id,
            revision_id,
            subvendor_id,
            subdevice_id,
        }
    }
}

#[derive(Debug)]
pub enum DeviceInfo {
    Pci(PCIDeviceInfo),
    Usb,
    Platform,
    Host1x,
}

#[derive(Debug)]
pub struct DrmDevice {
    //    nodes: [&str],
    available_nodes: i32,
    bus_type: DrmBus,
    bus_info: BusInfo,
    device_info: DeviceInfo,
}

impl DrmDevice {
    fn new(
        node_type: DrmNodeType,
        subsystem_type: DrmBus,
        bus_info: BusInfo,
        device_info: DeviceInfo,
    ) -> DrmDevice {
        DrmDevice {
            available_nodes: 1 << (node_type as i32),
            bus_type: subsystem_type,
            bus_info,
            device_info,
        }
    }
}

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

    match node_type {
        DrmNodeType::Primary => {
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
        _ => Err("Unsupported DRM Node Type")?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_device_happy_path() {
        let device = process_device("card0", Some(DrmBus::PCI)).unwrap();
        dbg!(device);
    }
}
