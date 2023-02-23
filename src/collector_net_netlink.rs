use crate::datastructs::Publisher;
use crate::publisher::ConcretePublisher;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;
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

pub struct NetworkSource {
    handle: netlink_proto::ConnectionHandle<RtnlMessage>,
    messages: UnboundedReceiver<(NetlinkMessage<RtnlMessage>, SocketAddr)>,
}

impl NetworkSource {
    pub fn new() -> Result<NetworkSource, io::Error> {
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

    pub async fn collect_publish_current(&mut self, publisher: &ConcretePublisher
    ) -> Result<(), Box<dyn Error>> {
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
        loop {
            if let Some(packet) = nl_response.next().await {
                if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = packet {
                    publish_rtnetlink(&publisher, &msg)?;
                }
                //println!("<<< {:?}", packet);
            } else {
                break;
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
        loop {
            if let Some(packet) = nl_response.next().await {
                if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = packet {
                    publish_rtnetlink(&publisher, &msg)?;
                }
                //println!("<<< {:?}", packet);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub async fn collect_publish_loop(&mut self, publisher: &ConcretePublisher
    ) -> Result<(), io::Error> {
        while let Some((message, _)) = self.messages.next().await {
            //println!("rtnetlink change message - {message:?}");
            if let NetlinkMessage{payload: NetlinkPayload::InnerMessage(msg), ..} = message {
                publish_rtnetlink(&publisher, &msg)?;
            }
        }

        Ok(())
    }
}

fn publish_rtnetlink(publisher: &ConcretePublisher, nl_msg: &RtnlMessage) -> Result<(), io::Error> {
    match nl_msg {
        RtnlMessage::NewLink(LinkMessage{header, nlas, ..}) => {
            //println!("{:?}", nlas);
            let mut address_bytes: Option<&Vec<u8>> = None;
            for nla in nlas {
                match nla {
                    link::nlas::Nla::Address(addr) => address_bytes = Some(addr),
                    _ => (),
                }
            }
            let mac_address = match address_bytes {
                Some(address_bytes) => address_bytes.iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<Vec<String>>().join(":"),
                None => "".to_string(),
            };

            // FIXME lookup the iface name from index
            let ifname = header.index.to_string();
            publisher.publish_net_iface_mac(&ifname, &mac_address)?;
        },
        RtnlMessage::NewAddress(address_msg) => {
            // FIXME does not distinguish when IP is on DOWN iface
            let (ifname, address) = nl_addressmessage_decode(address_msg)?;
            publisher.publish_net_iface_address(&ifname, &address)?;
        },
        RtnlMessage::DelAddress(address_msg) => {
            let (ifname, address) = nl_addressmessage_decode(address_msg)?;
            publisher.unpublish_net_iface_address(&ifname, &address)?;
        },
        _ => {
            println!("unhandled RtnlMessage: {:?}", nl_msg);
        },
    };
    Ok(())
}

fn nl_addressmessage_decode(msg: &AddressMessage) -> Result<(String, IpAddr), io::Error> {
    let AddressMessage{header, nlas, ..} = msg;
    //println!("{header:?} {nlas:?}");

    // extract fields of interest
    let mut address_bytes: Option<&Vec<u8>> = None;
    for nla in nlas {
        match nla {
            address::nlas::Nla::Address(addr) => address_bytes = Some(addr),
            _ => (),
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

    // FIXME lookup the iface name from index
    let ifname = header.index.to_string();

    match address {
        Some(address) => Ok((ifname, address)),
        None => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown address")),
    }
}
