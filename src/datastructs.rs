use std::net::IpAddr;

pub struct KernelInfo {
    pub release: String,
}

pub struct NetInterface {
    pub index: u32,
    pub name: String,
    pub vif_index: Option<u32>, // on Xen PV device only
}

pub enum NetEventOp {
    AddMac(String),
    RmMac(String),
    AddIp(IpAddr),
    RmIp(IpAddr),
}

pub struct NetEvent {
    pub iface: NetInterface,
    pub op: NetEventOp,
}
