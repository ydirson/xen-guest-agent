use std::error::Error;
use std::fmt::{Debug, Formatter, Result, Display};

pub enum XenError {
    NotInGuest,
    HypervisorNotXen,
}

impl Error for XenError {}

impl Debug for XenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

impl Display for XenError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            XenError::NotInGuest => write!(f, "Cannot identify hypervisor"),
            XenError::HypervisorNotXen => write!(f, "Hypervisor is not Xen"),
        }
    }
}
