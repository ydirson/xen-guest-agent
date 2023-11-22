use crate::datastructs::{NetEvent, ToolstackNetInterface};

pub fn get_toolstack_interface(iface_name: &str) -> ToolstackNetInterface {
    return ToolstackNetInterface::None;
}

pub fn add_vif_info(_event: &mut NetEvent) -> () {
}
