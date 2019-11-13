use crate::core::Result;

use crate::node::DrmNode;

#[derive(Debug, PartialEq)]
pub enum DrmBus {
    PCI = 0,
    USB = 1,
    PLATFORM = 2,
    HOST1X = 3,
    VIRTIO = 0x10,
}

impl DrmBus {
    pub fn get_subsystem_type(drm_node: &DrmNode) -> Result<DrmBus> {
        let subsystem_path = drm_node.get_subsystem_path();
        if let Some(link_file) = subsystem_path.file_name() {
            if let Some(link_file_name) = link_file.to_str() {
                return match link_file_name {
                    "pci" => Ok(DrmBus::PCI),
                    "usb" => Ok(DrmBus::USB),
                    "platform" => Ok(DrmBus::PLATFORM),
                    "spi" => Ok(DrmBus::PLATFORM),
                    "host1x" => Ok(DrmBus::HOST1X),
                    "virtio" => Ok(DrmBus::VIRTIO),
                    _ => Err(format!(
                        "Could not match {} to DRM Bus type",
                        link_file_name
                    ))?,
                };
            }
        }

        Err(format!("Seems like invalid drm node was provided.",))?
    }
}
