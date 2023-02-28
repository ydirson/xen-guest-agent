use crate::datastructs::{OsInfo, KernelInfo,
                         Publisher};
use std::error::Error;
use std::io;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub struct ConcretePublisher {
    xs: Xs,
}

impl ConcretePublisher {
    pub fn new() -> Result<ConcretePublisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        Ok(ConcretePublisher { xs })
    }
}

const PROTOCOL_VERSION: &str = "0.1.0";

impl Publisher for ConcretePublisher {

    fn publish_static(&self, os_info: &OsInfo, kernel_info: &KernelInfo) -> Result<(), io::Error> {
        xs_publish(&self.xs, "data/xen-guest-agent", PROTOCOL_VERSION)?;
        xs_publish(&self.xs, "data/os/name", &os_info.name)?;
        xs_publish(&self.xs, "data/os/version", &os_info.version)?;
        xs_publish(&self.xs, "data/os/class", "unix")?;
        xs_publish(&self.xs, "data/os/unix/kernel-version", &kernel_info.release)?;

        Ok(())
    }
}

fn xs_publish(xs: &Xs, key: &str, value: &str) -> Result<(), io::Error> {
    println!("W: {}={:?}", key, value);
    xs.write(XBTransaction::Null, key, value)
}
