mod datastructs;

#[cfg_attr(feature = "xenstore", path = "publisher_xenstore.rs")]
mod publisher;
#[cfg(feature = "xenstore")]
mod xenstore_schema_rfc;
#[cfg(feature = "xenstore")]
mod xenstore_schema_std;

#[cfg_attr(feature = "net_netlink", path = "collector_net_netlink.rs")]
#[cfg_attr(feature = "net_pnet", path = "collector_net_pnet.rs")]
mod collector_net;

#[cfg_attr(target_os = "linux", path = "collector_memory_linux.rs")]
#[cfg_attr(target_os = "freebsd", path = "collector_memory_bsd.rs")]
mod collector_memory;

#[cfg_attr(target_os = "linux", path = "vif_detect_linux.rs")]
#[cfg_attr(target_os = "freebsd", path = "vif_detect_freebsd.rs")]
mod vif_detect;

#[cfg_attr(target_os = "linux", path = "hypervisor_linux.rs")]
mod hypervisor;

mod error;

use clap::Parser;

use crate::collector_memory::MemorySource;
use crate::collector_net::NetworkSource;
use crate::datastructs::KernelInfo;
use crate::hypervisor::check_is_in_xen_guest;
use crate::publisher::Publisher;

use futures::{pin_mut, select, FutureExt, TryStreamExt};
use std::error::Error;
use std::io;
use std::str::FromStr;
use std::time::Duration;

const REPORT_INTERNAL_NICS: bool = false; // FIXME make this a CLI flag
const MEM_PERIOD_SECONDS: u64 = 60;
const DEFAULT_LOGLEVEL: &str = "info";


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    setup_logger(cli.stderr, &cli.loglevel)?;

    if let Err(err) = check_is_in_xen_guest() {
        log::error!("not starting xen-guest-agent, {err}");
        return Err(err.into())
    }

    let mut publisher = Publisher::new()?;

    let mut collector_memory = MemorySource::new()?;

    // Remove old entries from previous agent to avoid having unknown
    // interfaces. We will repopulate existing ones immediatly.
    publisher.cleanup_ifaces()?;

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
    let network_cache = Box::leak(Box::default());
    let mut collector_net = NetworkSource::new(network_cache)?;
    for event in collector_net.collect_current().await? {
        if REPORT_INTERNAL_NICS || ! event.iface.borrow().toolstack_iface.is_none() {
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
                    Some(event) => {
                        if REPORT_INTERNAL_NICS || ! event.iface.borrow().toolstack_iface.is_none() {
                            publisher.publish_netevent(&event)?;
                        } else {
                            log::debug!("no toolstack iface in {event:?}");
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

#[derive(clap::Parser)]
struct Cli {
    /// Print logs to stderr instead of system logs
    #[arg(short, long)]
    stderr: bool,

    /// Highest level of detail to log
    #[arg(short, long, default_value_t = String::from(DEFAULT_LOGLEVEL))]
    loglevel: String,
}

fn setup_logger(use_stderr:bool, loglevel_string: &str) -> Result<(), Box<dyn Error>> {
    if use_stderr {
        setup_env_logger(loglevel_string)?;
    } else {
        #[cfg(not(unix))]
        panic!("no system logger supported");

        #[cfg(unix)]
        setup_system_logger(loglevel_string)?;
    }
    Ok(())
}

// stdout logger for platforms with no specific implementation
fn setup_env_logger(loglevel_string: &str) -> Result<(), Box<dyn Error>> {
    // set default threshold to "info" not "error"
    let env = env_logger::Env::default().default_filter_or(loglevel_string);
    env_logger::Builder::from_env(env).init();
    Ok(())
}

#[cfg(unix)]
// syslog logger
fn setup_system_logger(loglevel_string: &str) -> Result<(), Box<dyn Error>> {
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
    log::set_max_level(log::LevelFilter::from_str(loglevel_string)?);
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
