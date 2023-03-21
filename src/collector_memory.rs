use std::io;

pub struct MemorySource {
}

impl MemorySource {
    pub fn new() -> io::Result<MemorySource> {
        Ok(MemorySource {})
    }

    pub fn get_total_kb(&mut self) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no implementation for mem_total"))
    }
    pub fn get_available_kb(&mut self) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no implementation for mem_avail"))
    }
}
