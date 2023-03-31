// default no-op Publisher implementation
use crate::datastructs::{KernelInfo, NetEvent};
use os_info;
use std::error::Error;
use std::io;

pub struct Publisher {
}

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        Ok(Publisher {})
    }

    pub fn publish_static(&self, os_info: &os_info::Info, _kernel_info: &Option<KernelInfo>,
                          _mem_total_kb: Option<usize>,
    ) -> io::Result<()> {
        Ok(())
    }
    pub fn publish_memfree(&mut self, _mem_free_kb: usize) -> io::Result<()> {
        Ok(())
    }
    pub fn publish_netevent(&self, _event: &NetEvent) -> io::Result<()> {
        Ok(())
    }
}
