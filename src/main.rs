use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

const PROTOCOL_VERSION: &str = "0.1.0";

fn main() -> Result<(), Box<dyn Error>> {
    let xs = Xs::new(XsOpenFlags::ReadOnly)?;

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    publish_static(&xs, &os_info, &kernel_info)?;

    Ok(())
}

struct OsInfo {
    name: String,
    version: String,
}

struct KernelInfo {
    release: String,
}

// /etc/os-release implementation
fn collect_os() -> Result<OsInfo, io::Error> {
    // empty default values, should not happen
    let mut name = "".to_string();
    let mut version = "".to_string();

    let file = File::open("/etc/os-release")?;
    for line_result in io::BufReader::new(file).lines() {
        match line_result {
            Ok(line) => { // FIXME not proper parsing of quoted shell string vars
                let v: Vec<&str> = line.split("=").collect();
                let (key, value) = (v[0], v[1]);
                let value = value.trim_matches('"').to_string();
                match key {
                    "NAME" => name = value,
                    "VERSION_ID" => version = value,
                    _ => (),
                };
            },
            Err(err) => return Err(err), // FIXME propagation
        }
    }

    let info = OsInfo {
        name,
        version,
    };

    Ok(info)
}

// UNIX uname() implementation
fn collect_kernel() -> Result<KernelInfo, io::Error> {
    let uname_info = uname::uname()?;
    let info = KernelInfo {
        release: uname_info.release,
    };

    Ok(info)
}

// (partial) xenstore implementation with XenServer layout
fn publish_static(xs: &Xs, os_info: &OsInfo,
                  kernel_info: &KernelInfo,
) -> Result<(), io::Error> {
    xs.write(XBTransaction::Null, "data/xen-guest-agent", PROTOCOL_VERSION)?;
    xs.write(XBTransaction::Null, "data/os/name", &os_info.name)?;
    xs.write(XBTransaction::Null, "data/os/version", &os_info.version)?;
    xs.write(XBTransaction::Null, "data/os/class", "unix")?;
    xs.write(XBTransaction::Null, "data/os/unix/kernel-version", &kernel_info.release)?;

    Ok(())
}
