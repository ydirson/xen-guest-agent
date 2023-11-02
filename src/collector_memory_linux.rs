use std::fs::File;
use std::io::{self, Read, Seek};

pub struct MemorySource {
    meminfo: File,
}

impl MemorySource {
    pub fn new() -> io::Result<MemorySource> {
        let meminfo = File::open("/proc/meminfo")?;
        Ok(MemorySource { meminfo })
    }

    pub fn get_total_kb(&mut self) -> io::Result<usize> {
        self.get_num_field("MemTotal:")
    }
    pub fn get_available_kb(&mut self) -> io::Result<usize> {
        self.get_num_field("MemAvailable:")
    }
    
    fn get_num_field(&mut self, tag: &str) -> io::Result<usize> {
        self.meminfo.rewind()?;
        let mut rawdata = String::new();
        self.meminfo.read_to_string(&mut rawdata)?;
        let tagindex = rawdata
            .find(tag)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData,
                                  format!("could not find {tag}")))?;
        let numindex = rawdata[tagindex + tag.len()..]
            .find(|c: char| c.is_ascii_digit())
            .ok_or(io::Error::new(io::ErrorKind::InvalidData,
                                  format!("no number after {tag}")))?;
        let num_start = tagindex + tag.len() + numindex;
        let num_end = num_start + (rawdata[num_start..]
                                   .find(|c: char| !c.is_ascii_digit())
                                   .unwrap());
        Ok(rawdata[num_start..num_end].parse().unwrap())
    }
}
