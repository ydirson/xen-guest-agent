use crate::datastructs::{KernelInfo, NetEvent};
use std::error::Error;
use std::io;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

// FIXME: this keeps the choice at compile-time
#[cfg(not(feature = "xenstore-rfc"))]
use crate::xenstore_schema_std::Schema;
#[cfg(feature = "xenstore-rfc")]
use crate::xenstore_schema_rfc::Schema;

pub trait XenstoreSchema {
    fn new(xs: Xs) -> Self where Self: Sized;
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
        let schema = Box::new(Schema::new(xs));
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

pub fn xs_publish(xs: &Xs, key: &str, value: &str) -> io::Result<()> {
    println!("W: {}={:?}", key, value);
    xs.write(XBTransaction::Null, key, value)
}

pub fn xs_unpublish(xs: &Xs, key: &str) -> io::Result<()> {
    println!("D: {}", key);
    xs.rm(XBTransaction::Null, key)
}
