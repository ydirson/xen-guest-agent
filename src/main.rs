mod datastructs;
#[cfg(unix)]
mod unix_helpers;

#[cfg_attr(feature = "xenstore", path = "publisher_xenstore.rs")]
mod publisher;
#[cfg(feature = "xenstore")]
mod xenstore_schema_std;
#[cfg(feature = "xenstore")]
mod xenstore_schema_rfc;

#[cfg_attr(feature = "net_netlink", path = "collector_net_netlink.rs")]
#[cfg_attr(feature = "net_pnet", path = "collector_net_pnet.rs")]
mod collector_net;

#[cfg_attr(target_os = "linux", path = "collector_memory_linux.rs")]
mod collector_memory;

#[cfg_attr(target_os = "linux", path = "vif_detect_linux.rs")]
#[cfg_attr(target_os = "freebsd", path = "vif_detect_freebsd.rs")]
mod vif_detect;

use crate::datastructs::KernelInfo;
use crate::publisher::Publisher;
use crate::collector_net::NetworkSource;
use crate::collector_memory::MemorySource;

use futures::{FutureExt, pin_mut, select, TryStreamExt};
use std::error::Error;
use std::io;
use std::time::Duration;

const REPORT_INTERNAL_NICS: bool = false; // FIXME make this a CLI flag
const MEM_PERIOD_SECONDS: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger()?;

    let mut publisher = Publisher::new()?;

    let mut collector_memory = MemorySource::new()?;

    let kernel_info = collect_kernel()?;
    let mem_total_kb = match collector_memory.get_total_kb() {
        Ok(mem_total_kb) => Some(mem_total_kb),
        Err(error) if error.kind() == io::ErrorKind::Unsupported
            => { log::warn!("Memory stats not supported");
                 None
            },
        // propagate errors other than io::ErrorKind::Unsupported
        Err(error) => Err(error)?,
    };
    publisher.publish_static(&os_info::get(), &kernel_info, mem_total_kb)?;

    // periodic memory stat
    let mut timer_stream = tokio::time::interval(Duration::from_secs(MEM_PERIOD_SECONDS));

    // network events
    let mut collector_net = NetworkSource::new()?;
    for mut event in collector_net.collect_current().await? {
        vif_detect::add_vif_info(&mut event);
        if REPORT_INTERNAL_NICS || ! event.iface.toolstack_iface.is_none() {
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
                        if REPORT_INTERNAL_NICS || ! event.iface.toolstack_iface.is_none() {
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
                    Err(ref e) if e.kind() == io::ErrorKind::Unsupported => (),
                    Err(e) => Err(e)?,
                }
            },
            complete => break,
        }
    }

    Ok(())
}

#[cfg(not(unix))]
// stdout logger for platforms with no specific implementation
fn setup_logger() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    Ok(())
}

#[cfg(unix)]
// syslog logger
fn setup_logger() -> Result<(), Box<dyn Error>> {
    let formatter = syslog::Formatter3164 {
        facility: syslog::Facility::LOG_USER,
        hostname: None,
        process: env!("CARGO_PKG_NAME").into(),
        pid: 0,
    };

    let logger = match syslog::unix(formatter) {
        Err(e) => { eprintln!("impossible to connect to syslog: {:?}", e); return Ok(()); },
        Ok(logger) => logger,
    };
    log::set_boxed_logger(Box::new(syslog::BasicLogger::new(logger)))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

// UNIX uname() implementation
#[cfg(unix)]
fn collect_kernel() -> io::Result<Option<KernelInfo>> {
    let uname_info = uname::uname()?;
    let info = KernelInfo {
        release: uname_info.release,
    };

    Ok(Some(info))
}

// default implementation
#[cfg(not(unix))]
fn collect_kernel() -> io::Result<Option<KernelInfo>> {
    Ok(None)
}
