use crate::datastructs::{OsInfo, KernelInfo, NetEvent, NetEventOp, NetInterface};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::IpAddr;
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

pub struct Publisher {
    xs: Xs,

    // use of integer indices for IP addresses requires to keep a mapping
    ip_addresses: IpList,
}

const NUM_IFACE_IPS: usize = 10;
type IfaceIpList = [Option<IpAddr>; NUM_IFACE_IPS];
struct IfaceIpStruct {
    v4: IfaceIpList,
    v6: IfaceIpList,
}
type IpList = HashMap<String, IfaceIpStruct>;


const AGENT_VERSION_MAJOR: &str = "1"; // XO does not show version at all if 0
const AGENT_VERSION_MINOR: &str = "0";
const AGENT_VERSION_MICRO: &str = "0"; // XAPI exposes "-1" if missing
const AGENT_VERSION_BUILD: &str = "proto"; // only place where we can be clear :)

impl Publisher {
    pub fn new() -> Result<Publisher, Box<dyn Error>> {
        let xs = Xs::new(XsOpenFlags::ReadOnly)?;
        let ip_addresses = IpList::new();
        Ok(Publisher { xs, ip_addresses })
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
    pub fn publish_netevent(&mut self, event: &NetEvent) -> io::Result<()> {
        let iface_id = match event.iface.vif_index {
            Some(id) => id,
            None => return Ok(()),
        };
        let xs_iface_prefix = format!("attr/vif/{iface_id}");
        match &event.op {
            NetEventOp::AddIp(address) => {
                let key_suffix = self.munged_address(address, &event.iface)?;
                xs_publish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"),
                           &address.to_string())?;
            },
            NetEventOp::RmIp(address) => {
                let key_suffix = self.munged_address(address, &event.iface)?;
                xs_unpublish(&self.xs, &format!("{xs_iface_prefix}/{key_suffix}"))?;
            },

            // FIXME extend IfaceIpStruct for this
            NetEventOp::AddMac(_mac_address) => {},
            NetEventOp::RmMac(_mac_address) => {},
        }
        Ok(())
    }


    fn munged_address(&mut self, addr: &IpAddr, iface: &NetInterface) -> io::Result<String> {
        let ip_entry = self.ip_addresses
            .entry(iface.name.clone()) // wtf, need cloning string for a lookup!?
            .or_insert(IfaceIpStruct{v4: [None; NUM_IFACE_IPS], v6: [None; NUM_IFACE_IPS]});
        let ip_list = match addr { IpAddr::V4(_) => &mut ip_entry.v4,
                                   IpAddr::V6(_) => &mut ip_entry.v6 };
        let ip_slot = get_ip_slot(addr, ip_list)?;
        match addr {
            IpAddr::V4(_) => Ok(format!("ipv4/{ip_slot}")),
            IpAddr::V6(_) => Ok(format!("ipv6/{ip_slot}")),
        }
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

fn get_ip_slot(ip: &IpAddr, list: &mut IfaceIpList) -> io::Result<usize> {
    let mut empty_idx: Option<usize> = None;
    for (idx, item) in list.iter().enumerate() {
        match item {
            Some(item) => if item == ip { return Ok(idx) }, // found
            None => if empty_idx.is_none() { empty_idx = Some(idx) }
        }
    }
    // not found, insert in empty space if possible
    if let Some(idx) = empty_idx {
        list[idx] = Some(*ip);
        return Ok(idx);
    }
    Err(io::Error::new(io::ErrorKind::OutOfMemory /*StorageFull?*/,
                       "no free slot for a new IP address"))
}
