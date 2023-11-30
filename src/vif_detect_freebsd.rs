use crate::datastructs::ToolstackNetInterface;

// identifies a VIF as named "xn%ID"

pub fn get_toolstack_interface(iface_name: &str) -> ToolstackNetInterface {
    const PREFIX: &str = "xn";
    if ! iface_name.starts_with(PREFIX) {
        log::debug!("ignoring interface {iface_name} as not starting with '{PREFIX}'");
        return ToolstackNetInterface::None;
    }
    match iface_name[PREFIX.len()..].parse() {
        Ok(index) => { return ToolstackNetInterface::Vif(index); },
        Err(e) => {
            log::error!("cannot parse a VIF number adter {PREFIX}: {e}");
            return ToolstackNetInterface::None;
        },
    }
}
