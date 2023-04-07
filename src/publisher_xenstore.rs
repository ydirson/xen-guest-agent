use crate::datastructs::{KernelInfo, NetEvent};
use std::env;
use std::error::Error;
use std::io;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub trait XenstoreSchema {
    fn publish_static(&self, os_info: &os_info::Info, kernel_info: &Option<KernelInfo>,
                      mem_total_kb: Option<usize>,
    ) -> io::Result<()>;
    fn publish_memfree(&self, mem_free_kb: usize) -> io::Result<()>;
    fn publish_netevent(&mut self, event: &NetEvent) -> io::Result<()>;
}

pub struct Publisher {
    schema: Box<dyn XenstoreSchema>,
}

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        let schema_name = env::var("XENSTORE_SCHEMA").unwrap_or("std".to_string());
        let schema_ctor = schema_from_name(&schema_name)?;
        let schema = schema_ctor(xs);
        Ok(Publisher { schema })
    }

    pub fn publish_static(&self, os_info: &os_info::Info, kernel_info: &Option<KernelInfo>,
                          mem_total_kb: Option<usize>,
    ) -> io::Result<()> {
        self.schema.publish_static(os_info, kernel_info, mem_total_kb)
    }
    pub fn publish_memfree(&mut self, mem_free_kb: usize) -> io::Result<()> {
        self.schema.publish_memfree(mem_free_kb)
    }
    pub fn publish_netevent(&mut self, event: &NetEvent) -> io::Result<()> {
        self.schema.publish_netevent(event)
    }
}

fn schema_from_name(name: &str) -> io::Result<&'static dyn Fn(Xs) -> Box<dyn XenstoreSchema>> {
    match name {
        "std" => Ok(&crate::xenstore_schema_std::Schema::new),
        "rfc" => Ok(&crate::xenstore_schema_rfc::Schema::new),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData,
                                format!("unknown schema '{name}'"))),
    }
}

pub fn xs_publish(xs: &Xs, key: &str, value: &str) -> io::Result<()> {
    println!("W: {}={:?}", key, value);
    xs.write(XBTransaction::Null, key, value)
}

pub fn xs_unpublish(xs: &Xs, key: &str) -> io::Result<()> {
    println!("D: {}", key);
    xs.rm(XBTransaction::Null, key)
}
