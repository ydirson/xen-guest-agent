mod datastructs;
mod helpers;

#[cfg_attr(feature = "xenstore", path = "publisher_xenstore.rs")]
mod publisher;

#[cfg_attr(feature = "netlink", path = "collector_net_netlink.rs")]
mod collector_net;

#[cfg_attr(target_os = "linux", path = "vif_detect_linux.rs")]
mod vif_detect;

use crate::datastructs::{OsInfo, KernelInfo};
use crate::publisher::Publisher;
use crate::collector_net::NetworkSource;

use futures::TryStreamExt;
use futures::pin_mut;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let publisher = Publisher::new()?;

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    publisher.publish_static(&os_info, &kernel_info)?;

    // network events
    let mut collector_net = NetworkSource::new()?;
    for mut event in collector_net.collect_current().await? {
        vif_detect::add_vif_info(&mut event);
        publisher.publish_netevent(&event)?;
    }
    let netevent_stream = collector_net.stream();
    pin_mut!(netevent_stream); // needed for iteration

    // main loop
    while let Some(mut event) = netevent_stream.try_next().await? {
        vif_detect::add_vif_info(&mut event);
        publisher.publish_netevent(&event)?;
    }

    Ok(())
}

// /etc/os-release implementation
fn collect_os() -> io::Result<OsInfo> {
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
fn collect_kernel() -> io::Result<KernelInfo> {
    let uname_info = uname::uname()?;
    let info = KernelInfo {
        release: uname_info.release,
    };

    Ok(info)
}
