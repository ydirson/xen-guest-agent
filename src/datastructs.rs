use std::io;

pub struct OsInfo {
    pub name: String,
    pub version: String,
}

pub struct KernelInfo {
    pub release: String,
}

// traits

pub trait Publisher {
    fn publish_static(&self, os_info: &OsInfo, kernel_info: &KernelInfo) -> Result<(), io::Error>;
}
