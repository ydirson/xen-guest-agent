use std::fs;
use crate::error::XenError;

pub fn check_is_in_xen_guest() -> Result<(), XenError> {
    match fs::read_to_string("/sys/hypervisor/type") {
        Ok(hypervisor_type) => {
            let hypervisor_type = hypervisor_type.trim();
            log::debug!("hypervisor_type {hypervisor_type}");
            if hypervisor_type.eq("xen") { Ok(()) } else { Err(XenError::HypervisorNotXen) }
        },
        Err(err) => {
            log::error!("could not identify hypervisor type, {err}");
            Err(XenError::NotInGuest)
        }
    }
}
