use crate::datastructs::{NetEvent, ToolstackNetInterface};
use std::fs;

// identifies a VIF from sysfs as devtype="vif", and take the VIF id
// from nodename="device/vif/$ID"

// FIXME does not attempt to detect sr-iov VIFs

pub fn add_vif_info(event: &mut NetEvent) {
    // FIXME: using ETHTOOL ioctl could be better
    let device_path = format!("/sys/class/net/{}/device", event.iface.name);
    if let Ok(devtype) = fs::read_to_string(format!("{device_path}/devtype")) {
        let devtype = devtype.trim();
        if devtype != "vif" {
            log::debug!("ignoring device {device_path}, devtype {devtype:?} not 'vif'");
            return;
        }
        if let Ok(nodename) = fs::read_to_string(format!("{device_path}/nodename")) {
            let nodename = nodename.trim();
            const PREFIX: &str = "device/vif/";
            if ! nodename.starts_with(PREFIX) {
                log::debug!("ignoring interface {nodename} as not under {PREFIX}");
                return;
            }
            let vif_id = nodename[PREFIX.len()..].parse().unwrap();
            event.iface.toolstack_iface = ToolstackNetInterface::Vif(vif_id);
        }
    }
}
