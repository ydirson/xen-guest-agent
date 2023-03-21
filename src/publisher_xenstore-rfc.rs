use crate::datastructs::{OsInfo, KernelInfo, NetEvent, NetEventOp};
use std::error::Error;
use std::io;
use std::net::IpAddr;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub struct Publisher {
    xs: Xs,
}

const PROTOCOL_VERSION: &str = "0.1.0";

// FIXME: this should be a runtime config of xenstore-std.rs

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        Ok(Publisher { xs })
    }

    pub fn publish_static(&self, os_info: &OsInfo, kernel_info: &KernelInfo,
                          _mem_total_kb: Option<usize>,
    ) -> io::Result<()> {
        xs_publish(&self.xs, "data/xen-guest-agent", PROTOCOL_VERSION)?;
        xs_publish(&self.xs, "data/os/name", &os_info.name)?;
        xs_publish(&self.xs, "data/os/version", &os_info.version)?;
        xs_publish(&self.xs, "data/os/class", "unix")?;
        xs_publish(&self.xs, "data/os/unix/kernel-version", &kernel_info.release)?;

        Ok(())
    }

    pub fn publish_memfree(&self, _mem_free_kb: usize) -> io::Result<()> {
        //xs_publish(&self.xs, "data/meminfo_free", &mem_free_kb.to_string())?;
        Ok(())
    }

    #[allow(clippy::useless_format)]
    pub fn publish_netevent(&self, event: &NetEvent) -> io::Result<()> {
        let iface_id = &event.iface.name;
        let xs_iface_prefix = format!("data/net/{iface_id}");
        match &event.op {
            NetEventOp::AddIp(address) => {
                let key_suffix = munged_address(address);
                xs_publish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"), "")?;
            },
            NetEventOp::RmIp(address) => {
                let key_suffix = munged_address(address);
                xs_unpublish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"))?;
            },
            NetEventOp::AddMac(mac_address) => {
                xs_publish(&self.xs, &format!("{xs_iface_prefix}"), mac_address)?;
            },
            NetEventOp::RmMac(_) => {
                xs_unpublish(&self.xs, &format!("{xs_iface_prefix}"))?;
            },
        }
        Ok(())
    }
}

fn xs_publish(xs: &Xs, key: &str, value: &str) -> io::Result<()> {
    println!("W: {}={:?}", key, value);
    xs.write(XBTransaction::Null, key, value)
}

fn xs_unpublish(xs: &Xs, key: &str) -> io::Result<()> {
    println!("D: {}", key);
    xs.rm(XBTransaction::Null, key)
}

fn munged_address(addr: &IpAddr) -> String {
    match addr {
        IpAddr::V4(addr) =>
            "ipv4/".to_string() + &addr.to_string().replace('.', "_"),
        IpAddr::V6(addr) =>
            "ipv6/".to_string() + &addr.to_string().replace(':', "_"),
    }
}
