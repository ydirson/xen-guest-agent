mod datastructs;
mod publisher_xenstore;

use crate::datastructs::{OsInfo, KernelInfo,
                         Publisher};
use crate::publisher_xenstore::ConcretePublisher;

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let publisher = ConcretePublisher::new()?;

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    publisher.publish_static(&os_info, &kernel_info)?;

    Ok(())
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
