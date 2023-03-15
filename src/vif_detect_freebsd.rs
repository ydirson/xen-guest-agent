use crate::datastructs::{NetEvent, ToolstackNetInterface};

// identifies a VIF as named "xn%ID"

pub fn add_vif_info(event: &mut NetEvent) -> () {
    const PREFIX: &str = "xn";
    if ! event.iface.name.starts_with(PREFIX) { return; }
    if let Ok(index) = event.iface.name[PREFIX.len()..].parse() {
        event.iface.toolstack_iface = ToolstackNetInterface::VIF(index);
    }
}
