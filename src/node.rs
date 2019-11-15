use crate::core::Result;

use std::os::linux::fs::MetadataExt;
use std::path::Path;

static DRM_DIR_NAME: &str = "/dev/dri";

static DRM_PRIMARY_MINOR_NAME: &str = "card";
static DRM_CONTROL_MINOR_NAME: &str = "controlD";
static DRM_RENDER_MINOR_NAME: &str = "renderD";

fn major(dev: u64) -> u64 {
    let mut major = (dev & 0x00000000000fff00) >> 8;
    major |= (dev & 0xfffff00000000000) >> 32;
    major
}

fn minor(dev: u64) -> u64 {
    let mut minor = (dev & 0x00000000000000ff) >> 0;
    minor |= (dev & 0x00000ffffff00000) >> 12;
    minor
}

#[derive(Debug, PartialEq)]
pub enum DrmNodeType {
    Primary = 0,
    Control = 1,
    Render = 2,
}

impl DrmNodeType {
    /// # Examples
    ///
    /// DRM Node types:
    ///
    /// ```
    /// let node_type = rust_drm::DrmNodeType::from_minor_name("card0").unwrap();
    /// assert_eq!(node_type, rust_drm::DrmNodeType::Primary);
    ///
    /// let node_type = rust_drm::DrmNodeType::from_minor_name("controlD128").unwrap();
    /// assert_eq!(node_type, rust_drm::DrmNodeType::Control);
    ///
    /// let node_type = rust_drm::DrmNodeType::from_minor_name("renderD128").unwrap();
    /// assert_eq!(node_type, rust_drm::DrmNodeType::Render);
    /// ```
    ///
    /// Unknown DRM Node type:
    ///
    /// ```
    /// assert!(rust_drm::DrmNodeType::from_minor_name("unknownD128").is_err());
    /// ```
    pub fn from_minor_name(name: &str) -> Result<DrmNodeType> {
        match name {
            s if s.starts_with(DRM_PRIMARY_MINOR_NAME) => Ok(DrmNodeType::Primary),
            s if s.starts_with(DRM_CONTROL_MINOR_NAME) => Ok(DrmNodeType::Control),
            s if s.starts_with(DRM_RENDER_MINOR_NAME) => Ok(DrmNodeType::Render),
            _ => Err(format!("Could not match {} to DRM Node Type", name))?,
        }
    }
}

pub struct DrmNode {
    major: u64,
    minor: u64,
}

impl DrmNode {
    pub fn from_device_name(device_name: &str) -> Result<DrmNode> {
        let node_path = Path::new(DRM_DIR_NAME).join(device_name);
        let meta = std::fs::metadata(node_path)?;
        let st_rdev = meta.st_rdev();

        Ok(DrmNode {
            major: major(st_rdev),
            minor: minor(st_rdev),
        })
    }

    pub fn device_dir_exists(&self) -> bool {
        let drm_device_dir_name = format!("/sys/dev/char/{}:{}/device/drm", self.major, self.minor);
        let drm_device_path = Path::new(&drm_device_dir_name);
        drm_device_path.exists() && drm_device_path.is_dir()
    }

    pub fn get_device_path(&self) -> std::path::PathBuf {
        let drm_device_dir_name = format!("/sys/dev/char/{}:{}/device", self.major, self.minor);
        Path::new(&drm_device_dir_name).canonicalize().unwrap()
    }

    pub fn get_subsystem_path(&self) -> std::path::PathBuf {
        let drm_device_dir_name = format!(
            "/sys/dev/char/{}:{}/device/subsystem",
            self.major, self.minor
        );
        Path::new(&drm_device_dir_name).canonicalize().unwrap()
    }

    pub fn get_config_path(&self) -> std::path::PathBuf {
        let drm_device_dir_name =
            format!("/sys/dev/char/{}:{}/device/config", self.major, self.minor);
        Path::new(&drm_device_dir_name).canonicalize().unwrap()
    }
}
