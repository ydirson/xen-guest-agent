use async_stream::try_stream;
use crate::datastructs::{NetEvent, NetEventOp, NetInterface, ToolstackNetInterface};
use futures::stream::Stream;
use ipnetwork::IpNetwork;
use pnet_base::MacAddr;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::time::Duration;

const IFACE_PERIOD_SECONDS: u64 = 60;

#[derive(Debug, Eq, Hash, PartialEq)]
enum Address {
    IP(IpNetwork),
    MAC(MacAddr),
}
struct InterfaceInfo {
    name: String,
    addresses: HashSet<Address>,
}

impl InterfaceInfo {
    pub fn new(name: &str) -> InterfaceInfo {
        InterfaceInfo { name: name.to_string(), addresses: HashSet::new() }
    }
}

type NetworkView = HashMap<u32, InterfaceInfo>;
pub struct NetworkSource {
    cache: NetworkView,
}

impl NetworkSource {
    pub fn new() -> io::Result<NetworkSource> {
        Ok(NetworkSource {cache: NetworkView::new()})
    }

    pub async fn collect_current(&mut self) -> Result<Vec<NetEvent>, Box<dyn Error>> {
        Ok(self.get_ifconfig_data()?)
    }

    pub fn stream(&mut self) -> impl Stream<Item = io::Result<NetEvent>> + '_ {
        try_stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(IFACE_PERIOD_SECONDS));
            loop {
                interval.tick().await;
                for net_event in self.get_ifconfig_data()? {
                     yield net_event;
                }
            }
        }
    }


    fn get_ifconfig_data(&mut self) -> io::Result<Vec<NetEvent>> {
        let network_interfaces = pnet_datalink::interfaces();

        // get a full view of interfaces, diffable with the cache
        let mut network_view: NetworkView = NetworkView::new();
        for iface in network_interfaces.iter() {
            // KLUDGE: drop ":alias" suffix for Linux interface aliases
            let name = iface.name.split(":").next().unwrap_or(&iface.name);
            let entry = network_view
                .entry(iface.index)
                .or_insert_with(|| InterfaceInfo::new(name));
            for ip in &iface.ips {
                entry.addresses.insert(Address::IP(*ip));
            }
            if let Some(mac) = iface.mac {
                entry.addresses.insert(Address::MAC(mac));
            }
        }

        // diff cache and view

        // events to be returned
        let mut events = vec!();
        // pseudo-const to get a valid reference for unwrap_or
        let empty_address_set: HashSet<Address> = HashSet::new();

        // disappearing addresses
        for (cached_iface_index, cached_info) in self.cache.iter() {
            let iface = NetInterface { index: *cached_iface_index,
                                       name: cached_info.name.to_string(),
                                       toolstack_iface: ToolstackNetInterface::None,
            };
            let iface_adresses =
                if let Some(iface_info) = network_view.get(cached_iface_index) {
                    &iface_info.addresses
                } else {
                    &empty_address_set
                };
            for disappearing in cached_info.addresses.difference(iface_adresses) {
                log::trace!("disappearing {}: {:?}", iface.name, disappearing);
                events.push(NetEvent{iface: iface.clone(),
                                     op: match disappearing {
                                         Address::IP(ip) => NetEventOp::RmIp(ip.ip()),
                                         Address::MAC(mac) => NetEventOp::RmMac((*mac).to_string()),
                                     }});
            }
        }
        // appearing addresses
        for (iface_index, iface_info) in network_view.iter() {
            let iface = NetInterface { index: *iface_index,
                                       name: iface_info.name.to_string(),
                                       toolstack_iface: ToolstackNetInterface::None,
            };
            let cache_adresses =
                if let Some(cache_info) = self.cache.get(iface_index) {
                    &cache_info.addresses
                } else {
                    &empty_address_set
                };
            for appearing in iface_info.addresses.difference(cache_adresses) {
                log::trace!("appearing {}: {:?}", iface.name, appearing);
                events.push(NetEvent{iface: iface.clone(),
                                     op: match appearing {
                                         Address::IP(ip) => NetEventOp::AddIp(ip.ip()),
                                         Address::MAC(mac) => NetEventOp::AddMac((*mac).to_string()),
                                     }});
            }

        }

        // replace cache with view
        self.cache = network_view; // FIXME expensive?

        Ok(events)
    }
}
