use async_stream::try_stream;
use crate::datastructs::{NetEvent, NetEventOp, NetInterface, ToolstackNetInterface};
use crate::unix_helpers::interface_name;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::{Stream, StreamExt};
use netlink_packet_core::{
    NetlinkHeader,
    NetlinkMessage,
    NetlinkPayload,
    NLM_F_DUMP,
    NLM_F_REQUEST,
};
use netlink_packet_route::{
    AddressMessage, address,
    LinkMessage, link,
    RtnlMessage,
};
use netlink_proto::{
    self, new_connection,
    sys::{protocols::NETLINK_ROUTE, AsyncSocket, SocketAddr},
};
use rtnetlink::constants::{
    RTMGRP_IPV4_IFADDR,
    RTMGRP_IPV6_IFADDR,
    RTMGRP_LINK,
    };
use std::convert::TryInto;
use std::error::Error;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::vec::Vec;

pub struct NetworkSource {
    handle: netlink_proto::ConnectionHandle<RtnlMessage>,
    messages: UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
}

impl NetworkSource {
    pub fn new() -> io::Result<NetworkSource> {
        let (mut connection, handle, messages) = new_connection(NETLINK_ROUTE)?;
        // What kinds of broadcast messages we want to listen for.
        let nl_mgroup_flags = RTMGRP_LINK | RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR;
        let nl_addr = SocketAddr::new(0, nl_mgroup_flags);
        connection
            .socket_mut()
            .socket_mut()
            .bind(&nl_addr)
            .expect("failed to bind");
        tokio::spawn(connection);
        Ok(NetworkSource { handle, messages })
    }

    pub async fn collect_current(&mut self) -> Result<Vec<NetEvent>, Box<dyn Error>> {
        let mut events = Vec::<NetEvent>::new();

        // Create the netlink message that requests the links to be dumped
        let mut nl_hdr = NetlinkHeader::default();
        nl_hdr.flags = NLM_F_DUMP | NLM_F_REQUEST;
        let nl_msg = NetlinkMessage::new(
            nl_hdr,
            RtnlMessage::GetLink(LinkMessage::default()).into(),
        );
        // Send the request
        let mut nl_response = self.handle.request(nl_msg, SocketAddr::new(0, 0))?;
        // Handle response
        while let Some(packet) = nl_response.next().await {
            if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = packet {
                events.push(self.netevent_from_rtnetlink(&msg)?);
            }
        }

        // Create the netlink message that requests the addresses to be dumped
        let mut nl_hdr = NetlinkHeader::default();
        nl_hdr.flags = NLM_F_DUMP | NLM_F_REQUEST;
        let nl_msg = NetlinkMessage::new(
            nl_hdr,
            RtnlMessage::GetAddress(AddressMessage::default()).into(),
        );
        // Send the request
        let mut nl_response = self.handle.request(nl_msg, SocketAddr::new(0, 0))?;
        // Handle response
        while let Some(packet) = nl_response.next().await {
            if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = packet {
                events.push(self.netevent_from_rtnetlink(&msg)?);
            }
        }

        Ok(events)
    }

    pub fn stream(&mut self) -> impl Stream<Item = io::Result<NetEvent>> + '_ {
        try_stream! {
            while let Some((message, _)) = self.messages.next().await {
                if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = message {
                    yield self.netevent_from_rtnetlink(&msg)?;
                }
            };
        }
    }

    fn netevent_from_rtnetlink(&mut self, nl_msg: &RtnlMessage) -> io::Result<NetEvent> {
        let event = match nl_msg {
            RtnlMessage::NewLink(link_msg) => {
                let (iface, mac_address) = self.nl_linkmessage_decode(link_msg)?;
                log::debug!("NewLink({iface:?} {mac_address})");
                NetEvent{iface, op: NetEventOp::AddMac(mac_address)}
            },
            RtnlMessage::DelLink(link_msg) => {
                let (iface, mac_address) = self.nl_linkmessage_decode(link_msg)?;
                log::debug!("DelLink({iface:?} {mac_address})");
                NetEvent{iface, op: NetEventOp::RmMac(mac_address)}
            },
            RtnlMessage::NewAddress(address_msg) => {
                // FIXME does not distinguish when IP is on DOWN iface
                let (iface, address) = self.nl_addressmessage_decode(address_msg)?;
                log::debug!("NewAddress({iface:?} {address})");
                NetEvent{iface, op: NetEventOp::AddIp(address)}
            },
            RtnlMessage::DelAddress(address_msg) => {
                let (iface, address) = self.nl_addressmessage_decode(address_msg)?;
                log::debug!("DelAddress({iface:?} {address})");
                NetEvent{iface, op: NetEventOp::RmIp(address)}
            },
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                          format!("unhandled RtnlMessage: {nl_msg:?}")));
            },
        };
        Ok(event)
    }

    fn nl_linkmessage_decode(&mut self, msg: &LinkMessage) -> io::Result<(NetInterface, String)> {
        let LinkMessage{header, nlas, ..} = msg;

        // extract fields of interest
        let mut iface_name: Option<String> = None;
        let mut address_bytes: Option<&Vec<u8>> = None;
        for nla in nlas {
            if let link::nlas::Nla::IfName(name) = nla {
                iface_name = Some(name.to_string());
            }
            if let link::nlas::Nla::Address(addr) = nla {
                address_bytes = Some(addr);
            }
        }
        // make sure message contains an address
        let mac_address = address_bytes.map(|address_bytes| address_bytes.iter()
                                            .map(|b| format!("{b:02x}"))
                                            .collect::<Vec<String>>().join(":"));

        let iface = NetInterface { index: header.index,
                                   name: iface_name.unwrap_or(String::from("")),
                                   toolstack_iface: ToolstackNetInterface::None,
        };

        match mac_address {
            Some(mac_address) => Ok((iface, mac_address)),
            None => Ok((iface, "".to_string())), // FIXME ad-hoc ugly, use Option<String> instead
        }
    }

    fn nl_addressmessage_decode(&mut self, msg: &AddressMessage) -> io::Result<(NetInterface, IpAddr)> {
        let AddressMessage{header, nlas, ..} = msg;

        // extract fields of interest
        let mut address_bytes: Option<&Vec<u8>> = None;
        for nla in nlas {
            if let address::nlas::Nla::Address(addr) = nla {
                address_bytes = Some(addr);
                break;
            }
        }

        let address = match header.family {
            // PF_INET
            2  => match address_bytes {
                Some(address_bytes) => {
                    let address_array: [u8; 4] = address_bytes[..].try_into()
                        .expect("IPv4 with incorrect length");
                    Some(IpAddr::V4(Ipv4Addr::from(address_array)))
                },
                None => None,
            },
            // PF_INET6
            10 => match address_bytes {
                Some(address_bytes) => {
                    let address_array: [u8; 16] = address_bytes[..].try_into()
                        .expect("IPv6 with incorrect length");
                    Some(IpAddr::V6(Ipv6Addr::from(address_array)))
                },
                None => None,
            },
            _ => None,
        };

        let iface = NetInterface { index: header.index,
                                   name: interface_name(header.index),
                                   toolstack_iface: ToolstackNetInterface::None,
        };

        match address {
            Some(address) => Ok((iface, address)),
            None => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown address")),
        }
    }

}
