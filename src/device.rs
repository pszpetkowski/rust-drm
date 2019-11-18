use crate::bus::DrmBus;
use crate::node::{DrmNode, DrmNodeType};

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
    pub fn new(pci_slot_name: &str) -> PCIBusInfo {
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

pub struct PCIDeviceInfo {
    vendor_id: u16,
    device_id: u16,
    revision_id: u8,
    subvendor_id: u16,
    subdevice_id: u16,
}

impl std::fmt::Debug for PCIDeviceInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("PCIDeviceInfo")
            .field("vendor_id", &format_args!("0x{:x}", self.vendor_id))
            .field("device_id", &format_args!("0x{:x}", self.device_id))
            .field("revision_id", &self.revision_id)
            .field("subvendor_id", &self.subvendor_id)
            .field("subdevice_id", &self.subdevice_id)
            .finish()
    }
}

impl PCIDeviceInfo {
    pub fn new(drm_node: &DrmNode) -> PCIDeviceInfo {
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
    pub fn new(
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
