use std::net::IpAddr;

pub struct KernelInfo {
    pub release: String,
}

#[non_exhaustive]
#[derive(Clone)]
pub enum ToolstackNetInterface {
    None,
    VIF(u32),
    // SRIOV,
    // PciPassthrough,
    // UsbPassthrough,
}

impl ToolstackNetInterface {
    pub fn is_none(&self) -> bool {
        if let ToolstackNetInterface::None = self {
            return true;
        }
        return false;
    }
}

#[derive(Clone)]
pub struct NetInterface {
    pub index: u32,
    pub name: String,
    pub toolstack_iface: ToolstackNetInterface,
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
