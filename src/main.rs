mod datastructs;
mod helpers;

#[cfg_attr(feature = "xenstore", path = "publisher_xenstore-std.rs")]
#[cfg_attr(feature = "xenstore-rfc", path = "publisher_xenstore-rfc.rs")]
mod publisher;

#[cfg_attr(feature = "netlink", path = "collector_net_netlink.rs")]
mod collector_net;

#[cfg_attr(target_os = "linux", path = "collector_memory_linux.rs")]
mod collector_memory;

#[cfg_attr(target_os = "linux", path = "vif_detect_linux.rs")]
mod vif_detect;

use crate::datastructs::{OsInfo, KernelInfo};
use crate::publisher::Publisher;
use crate::collector_net::NetworkSource;
use crate::collector_memory::MemorySource;

use futures::{FutureExt, pin_mut, select, TryStreamExt};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::time::Duration;

const ONLY_VIF: bool = true;    // FIXME make this a CLI flag
const MEM_PERIOD_SECONDS: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut publisher = Publisher::new()?;

    let mut collector_memory = MemorySource::new()?;

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    let mem_total_kb = match collector_memory.get_total_kb() {
        Ok(mem_total_kb) => Some(mem_total_kb),
        Err(error) => { println!("No memory stats: {error}");
                        None
        },
        // FIXME should propagate errors other than io::ErrorKind::Unsupported
    };
    publisher.publish_static(&os_info, &kernel_info, mem_total_kb)?;

    // periodic memory stat
    let mut timer_stream = tokio::time::interval(Duration::from_secs(MEM_PERIOD_SECONDS));

    // network events
    let mut collector_net = NetworkSource::new()?;
    for mut event in collector_net.collect_current().await? {
        vif_detect::add_vif_info(&mut event);
        if ! (ONLY_VIF && event.iface.vif_index.is_none()) {
            publisher.publish_netevent(&event)?;
        }
    }
    let netevent_stream = collector_net.stream();
    pin_mut!(netevent_stream); // needed for iteration

    // main loop
    loop {
        select! {
            event = netevent_stream.try_next().fuse() => {
                match event? {
                    Some(mut event) => {
                        vif_detect::add_vif_info(&mut event);
                        if ! (ONLY_VIF && event.iface.vif_index.is_none()) {
                            publisher.publish_netevent(&event)?;
                        }
                    },
                    // FIXME can't we handle those in `select!` directly?
                    None => { /* closed? */ },
                };
            },
            _ = timer_stream.tick().fuse() => {
                match collector_memory.get_available_kb() {
                    Ok(mem_avail_kb) => publisher.publish_memfree(mem_avail_kb)?,
                    Err(_) => (),
                    // FIXME should propagate errors other than io::ErrorKind::Unsupported
                }
            },
            complete => break,
        }
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
