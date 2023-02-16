use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use xenstore_rs::{Xs, XsOpenFlags, XBTransaction};

fn main() -> Result<(), Box<dyn Error>> {
    let xs = Xs::new(XsOpenFlags::ReadOnly)?;

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    publish_static(&xs, &os_info, &kernel_info)?;

    Ok(())
}

struct OsInfo {
    pretty_name: String,
    nickname: String,
    version: String,
}

struct KernelInfo {
    release: String,
}

// /etc/os-release implementation
fn collect_os() -> Result<OsInfo, io::Error> {
    // arbitrary default values, should not happen
    let mut nickname = "undefined-os".to_string();
    let mut pretty_name = "Undefined Operating System".to_string();
    let mut version = "".to_string();

    let file = File::open("/etc/os-release")?;
    for line_result in io::BufReader::new(file).lines() {
        match line_result {
            Ok(line) => { // FIXME not proper parsing of quoted shell string vars
                let v: Vec<&str> = line.split("=").collect();
                let (key, value) = (v[0], v[1]);
                let value = value.trim_matches('"').to_string();
                match key {
                    "NAME" => nickname = value,
                    "PRETTY_NAME" => pretty_name = value,
                    "VERSION" => version = value,
                    _ => (),
                };
            },
            Err(err) => return Err(err), // FIXME propagation
        }
    }

    let info = OsInfo {
        pretty_name,
        nickname,
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
    xs.write(XBTransaction::Null, "data/os_name", &os_info.pretty_name)?;
    xs.write(XBTransaction::Null, "data/os_distro", &os_info.nickname)?;

    xs.write(XBTransaction::Null, "data/os_uname", &kernel_info.release)?;

    Ok(())
}
