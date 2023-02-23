use std::io;
use std::net::IpAddr;

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
    fn publish_net_iface_address(&self, ifname: &str, address: &IpAddr) -> Result<(), io::Error>;
    fn publish_net_iface_mac(&self, ifname: &str, mac_address: &str) -> Result<(), io::Error>;
}
