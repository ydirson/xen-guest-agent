// default no-op Publisher implementation
use crate::datastructs::{KernelInfo, NetEvent, NetEventOp};
use os_info;
use std::error::Error;
use std::io;

pub struct Publisher {
}

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        Ok(Publisher {})
    }

    pub fn publish_static(&self, os_info: &os_info::Info, kernel_info: &Option<KernelInfo>,
                          mem_total_kb: Option<usize>,
    ) -> io::Result<()> {
        println!("OS: {} - Version: {}", os_info.os_type(), os_info.version());
        if let Some(mem_total_kb) = mem_total_kb {
            println!("Total memory: {mem_total_kb} KB");
        }
        if let Some(KernelInfo{release}) = kernel_info {
            println!("Kernel version: {}", release);
        }
        Ok(())
    }
    pub fn publish_memfree(&mut self, mem_free_kb: usize) -> io::Result<()> {
        println!("Free memory: {mem_free_kb} KB");
        Ok(())
    }
    pub fn publish_netevent(&self, event: &NetEvent) -> io::Result<()> {
        let iface_id = &event.iface.name;
        match &event.op {
            NetEventOp::AddIp(address) => println!("{iface_id} +IP  {address}"),
            NetEventOp::RmIp(address) => println!("{iface_id} -IP  {address}"),
            NetEventOp::AddMac(mac_address) => println!("{iface_id} +MAC {mac_address}"),
            NetEventOp::RmMac(mac_address) => println!("{iface_id} -MAC {mac_address}"),
            _ =>
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                          format!("unhandled NetEvent: {event:?}"))),
        }
        Ok(())
    }
}
