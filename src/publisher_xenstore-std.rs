use crate::datastructs::{OsInfo, KernelInfo, NetEvent, NetEventOp};
use std::error::Error;
use std::io;
use std::net::IpAddr;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub struct Publisher {
    xs: Xs,
}

const AGENT_VERSION_MAJOR: &str = "1"; // XO does not show version at all if 0
const AGENT_VERSION_MINOR: &str = "0";
const AGENT_VERSION_MICRO: &str = "0"; // XAPI exposes "-1" if missing
const AGENT_VERSION_BUILD: &str = "proto"; // only place where we can be clear :)

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        Ok(Publisher { xs })
    }

    pub fn publish_static(&self, os_info: &OsInfo, kernel_info: &KernelInfo
    ) -> io::Result<()> {
        // FIXME this is not anywhere standard, just minimal XS compatibility
        xs_publish(&self.xs, "attr/PVAddons/MajorVersion", AGENT_VERSION_MAJOR)?;
        xs_publish(&self.xs, "attr/PVAddons/MinorVersion", AGENT_VERSION_MINOR)?;
        xs_publish(&self.xs, "attr/PVAddons/MicroVersion", AGENT_VERSION_MICRO)?;
        xs_publish(&self.xs, "attr/PVAddons/BuildVersion", AGENT_VERSION_BUILD)?;

        xs_publish(&self.xs, "data/os_name", &os_info.name)?;
        // FIXME .version only has "major" component right now; not a
        // big deal for a proto, os_minorver is known to be unreliable
        // in xe-guest-utilities at least for Debian
        xs_publish(&self.xs, "data/os_majorver", &os_info.version)?;
        xs_publish(&self.xs, "data/os_minorver", "0")?;
        xs_publish(&self.xs, "data/os_uname", &kernel_info.release)?;

        Ok(())
    }

    // see https://xenbits.xen.org/docs/unstable/misc/xenstore-paths.html#domain-controlled-paths
    pub fn publish_netevent(&self, event: &NetEvent) -> io::Result<()> {
        let iface_id = match event.iface.vif_index {
            Some(id) => id,
            None => return Ok(()),
        };
        let xs_iface_prefix = format!("attr/vif/{iface_id}");
        match &event.op {
            NetEventOp::AddIp(address) => {
                let key_suffix = munged_address(address);
                xs_publish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"),
                           &address.to_string())?;
            },
            NetEventOp::RmIp(address) => {
                let key_suffix = munged_address(address);
                xs_unpublish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"))?;
            },
            NetEventOp::AddMac(mac_address) => {
                let key_suffix = munged_mac_address(mac_address);
                xs_publish(&self.xs, &format!("{xs_iface_prefix}/mac/{key_suffix}"), &mac_address)?;
            },
            NetEventOp::RmMac(mac_address) => {
                let key_suffix = munged_mac_address(mac_address);
                xs_unpublish(&self.xs, &format!("{xs_iface_prefix}/mac/{key_suffix}"))?;
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
            "ipv4/".to_string() + &addr.to_string().replace(".", "_"),
        IpAddr::V6(addr) =>
            "ipv6/".to_string() + &addr.to_string().replace(":", "_"),
    }
}

fn munged_mac_address(addr: &str) -> String {
    addr.to_string().replace(":", "_")
}
