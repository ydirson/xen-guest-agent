// default no-op Publisher implementation
use crate::datastructs::{OsInfo, KernelInfo, NetEvent};
use std::error::Error;
use std::io;

pub struct Publisher {
}

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        Ok(Publisher {})
    }

    pub fn publish_static(&self, _os_info: &OsInfo, _kernel_info: &KernelInfo
    ) -> io::Result<()> {
        Ok(())
    }
    pub fn publish_netevent(&self, event: &NetEvent) -> io::Result<()> {
        Ok(())
    }
}
