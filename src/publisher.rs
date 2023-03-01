// default no-op Publisher implementation
use crate::datastructs::{OsInfo, KernelInfo,
                         Publisher};
use std::error::Error;
use std::io;
use std::net::IpAddr;

pub struct ConcretePublisher {
}

impl ConcretePublisher {
    pub fn new() -> Result<ConcretePublisher, Box<dyn Error>> {
        Ok(ConcretePublisher {})
    }
}

impl Publisher for ConcretePublisher {

    fn publish_static(&self, _os_info: &OsInfo,
                      _kernel_info: &KernelInfo) -> Result<(), io::Error> {
        Ok(())
    }
    fn publish_net_iface_address(&self, _ifname: &str,
                                 _address: &IpAddr) -> Result<(), io::Error> {
        Ok(())
    }
}
