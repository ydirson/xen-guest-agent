use crate::datastructs::{OsInfo, KernelInfo,
                         Publisher};
use std::error::Error;
use std::io;
use std::net::IpAddr;
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

    fn publish_net_iface_address(&self, ifname: &str, address: &IpAddr) -> Result<(), io::Error> {
        let key_suffix = munged_address(address);
        xs_publish(&self.xs, &format!("data/net/{ifname}/{key_suffix}"), "")?;

        Ok(())
    }

    fn unpublish_net_iface_address(&self, ifname: &str, address: &IpAddr) -> Result<(), io::Error> {
        let key_suffix = munged_address(address);
        xs_unpublish(&self.xs, &format!("data/net/{ifname}/{key_suffix}"))?;

        Ok(())
    }

    fn publish_net_iface_mac(&self, ifname: &str, mac_address: &str) -> Result<(), io::Error> {
        xs_publish(&self.xs, &format!("data/net/{ifname}"), &mac_address)?;

        Ok(())
    }

    fn unpublish_net_iface_mac(&self, ifname: &str, _mac_address: &str) -> Result<(), io::Error> {
        xs_unpublish(&self.xs, &format!("data/net/{ifname}"))?;

        Ok(())
    }
}

fn xs_publish(xs: &Xs, key: &str, value: &str) -> Result<(), io::Error> {
    println!("W: {}={:?}", key, value);
    xs.write(XBTransaction::Null, key, value)
}

fn xs_unpublish(xs: &Xs, key: &str) -> Result<(), io::Error> {
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
