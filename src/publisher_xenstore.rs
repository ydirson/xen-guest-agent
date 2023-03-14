use crate::datastructs::{OsInfo, KernelInfo, NetEvent, NetEventOp};
use std::error::Error;
use std::io;
use std::net::IpAddr;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub struct Publisher {
    xs: Xs,
}

const PROTOCOL_VERSION: &str = "0.1.0";

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        Ok(Publisher { xs })
    }

    pub fn publish_static(&self, os_info: &OsInfo, kernel_info: &KernelInfo
    ) -> io::Result<()> {
        xs_publish(&self.xs, "data/xen-guest-agent", PROTOCOL_VERSION)?;
        xs_publish(&self.xs, "data/os/name", &os_info.name)?;
        xs_publish(&self.xs, "data/os/version", &os_info.version)?;
        xs_publish(&self.xs, "data/os/class", "unix")?;
        xs_publish(&self.xs, "data/os/unix/kernel-version", &kernel_info.release)?;

        Ok(())
    }

    pub fn publish_netevent(&self, event: &NetEvent) -> io::Result<()> {
        match event {
            NetEvent{iface, op: NetEventOp::AddIp(address)} => {
                let iface_id = &iface.name;
                let key_suffix = munged_address(address);
                xs_publish(&self.xs, &format!("data/net/{iface_id}/{key_suffix}"),
                           "")?;
            },
            NetEvent{iface, op: NetEventOp::RmIp(address)} => {
                let iface_id = &iface.name;
                let key_suffix = munged_address(address);
                xs_unpublish(&self.xs, &format!("data/net/{iface_id}/{key_suffix}"))?;
            },
            NetEvent{iface, op: NetEventOp::AddMac(mac_address)} => {
                let iface_id = &iface.name;
                xs_publish(&self.xs, &format!("data/net/{iface_id}"), &mac_address)?;
            },
            NetEvent{iface, op: NetEventOp::RmMac(_)} => {
                let iface_id = &iface.name;
                xs_unpublish(&self.xs, &format!("data/net/{iface_id}"))?;
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
