use std::net::IpAddr;

pub struct OsInfo {
    pub name: String,
    pub version: String,
}

pub struct KernelInfo {
    pub release: String,
}

pub struct NetInterface {
    pub index: u32,
    pub name: String,
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
