mod datastructs;

#[cfg_attr(feature = "xenstore", path = "publisher_xenstore.rs")]
mod publisher;

use crate::datastructs::{OsInfo, KernelInfo,
                         Publisher};
use crate::publisher::ConcretePublisher;
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
    new_connection,
    sys::{protocols::NETLINK_ROUTE, AsyncSocket, SocketAddr},
};

use std::convert::TryInto;
use std::error::Error;
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let publisher = ConcretePublisher::new()?;

    let (mut nl_connection,
         mut nl_handle,
         _) = new_connection(NETLINK_ROUTE)?;
    let nl_addr = SocketAddr::new(0, 0);
    nl_connection
        .socket_mut()
        .socket_mut()
        .bind(&nl_addr)
        .expect("failed to bind");
    tokio::spawn(nl_connection);

    let os_info = collect_os()?;
    let kernel_info = collect_kernel()?;
    publisher.publish_static(&os_info, &kernel_info)?;

    // Create the netlink message that requests the links to be dumped
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_DUMP | NLM_F_REQUEST;
    let nl_msg = NetlinkMessage::new(
        nl_hdr,
        RtnlMessage::GetLink(LinkMessage::default()).into(),
    );
    // Send the request
    let mut nl_response = nl_handle.request(nl_msg, SocketAddr::new(0, 0))?;
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
    let mut nl_response = nl_handle.request(nl_msg, SocketAddr::new(0, 0))?;
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
            let (ifname, address) = nl_addressmessage_decode(address_msg)?;
            publisher.publish_net_iface_address(&ifname, &address)?;
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
