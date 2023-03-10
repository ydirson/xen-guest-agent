// default no-op Publisher implementation
use crate::datastructs::{OsInfo, KernelInfo, NetInterface};
use std::error::Error;
use std::io;
use std::net::IpAddr;

pub struct Publisher {
}

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        Ok(Publisher {})
    }

    pub fn publish_static(&self, _os_info: &OsInfo, _kernel_info: &KernelInfo
    ) -> Result<(), io::Error> {
        Ok(())
    }
    pub fn publish_net_iface_address(&self, _iface: &NetInterface, _address: &IpAddr
    ) -> Result<(), io::Error> {
        Ok(())
    }
    pub fn unpublish_net_iface_address(&self, _iface: &NetInterface, _address: &IpAddr
    ) -> Result<(), io::Error> {
        Ok(())
    }
    pub fn publish_net_iface_mac(&self, _iface: &NetInterface, _mac_address: &str
    ) -> Result<(), io::Error> {
        Ok(())
    }
    pub fn unpublish_net_iface_mac(&self, _iface: &NetInterface, _mac_address: &str
    ) -> Result<(), io::Error> {
        Ok(())
    }
}
