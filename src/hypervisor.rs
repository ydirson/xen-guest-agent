use crate::error::XenError;

pub fn check_is_in_xen_guest() -> Result<(), XenError> {
    // NOTE: 'Ok' here implies that we do not know how to check for the hypervisor,
    //  so we assume the users are aware of their actions.
    Ok(())
}
