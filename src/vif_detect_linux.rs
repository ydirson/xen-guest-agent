use crate::datastructs::ToolstackNetInterface;
use std::fs;

// identifies a VIF from sysfs as devtype="vif", and take the VIF id
// from nodename="device/vif/$ID"

// FIXME does not attempt to detect sr-iov VIFs

pub fn get_toolstack_interface(iface_name: &str) -> ToolstackNetInterface {
    // FIXME: using ETHTOOL ioctl could be better
    let device_path = format!("/sys/class/net/{}/device", iface_name);
    match fs::read_to_string(format!("{device_path}/devtype")) {
        Ok(devtype) => {
            let devtype = devtype.trim();
            if devtype != "vif" {
                log::debug!("ignoring device {device_path}, devtype {devtype:?} not 'vif'");
                return ToolstackNetInterface::None;
            }
            match fs::read_to_string(format!("{device_path}/nodename")) {
                Ok(nodename) => {
                    let nodename = nodename.trim();
                    const PREFIX: &str = "device/vif/";
                    if ! nodename.starts_with(PREFIX) {
                        log::debug!("ignoring interface {nodename} as not under {PREFIX}");
                        return ToolstackNetInterface::None;
                    }
                    let vif_id = nodename[PREFIX.len()..].parse().unwrap();
                    return ToolstackNetInterface::Vif(vif_id);
                },
                Err(e) => {
                    log::error!("reading {device_path}/nodename: {e}");
                    return ToolstackNetInterface::None;
                },
            }
        },
        Err(e) => {
            log::debug!("reading {device_path}/devtype: {e}");
            return ToolstackNetInterface::None;
        },
    }
}
