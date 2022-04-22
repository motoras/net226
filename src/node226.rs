use crate::frame::*;
use bytes::{BufMut, BytesMut};
use log::{debug, info, trace};
use message_io::{
    network::{NetEvent, RemoteAddr, ToRemoteAddr, Transport},
    node::{NodeEvent, NodeHandler, NodeListener},
};
use std::fmt::{Display, Formatter, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum Signals226 {
    Heartbeat,
    Quit,
    ShutdownNow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node226Info {
    uuid: Uuid,
    last_heart_beat: SystemTime,
    home_address: SocketAddr,
}

#[derive(Debug, Clone, Copy)]
pub struct Node226 {
    uuid: Uuid,
    group_addr: SocketAddr,
    heart_beat: Duration,
}

impl Node226 {
    fn new(group_addr: SocketAddr) -> Node226 {
        assert!(group_addr.ip().is_multicast());
        Node226 {
            uuid: Uuid::new_v4(),
            group_addr,
            heart_beat: Duration::from_secs(10),
        }
    }

    fn join_msg(&self, port: u16, flags: u8) -> BytesMut {
        let mut buf = BytesMut::with_capacity(22);
        buf.put_u16(JOIN);
        buf.put_u128(self.uuid.as_u128());
        buf.put_u16(port);
        buf.put_u8(flags);
        buf.put_u8(0u8);
        buf
    }

    fn hb_msg(&self, port: u16, flags: u8) -> BytesMut {
        let mut buf = BytesMut::with_capacity(22);
        buf.put_u16(HEARTBEAT);
        buf.put_u128(self.uuid.as_u128());
        buf.put_u16(port);
        buf.put_u8(flags);
        buf.put_u8(0u8);
        buf
    }

    fn leave_msg(&self, flags: u8) -> BytesMut {
        let mut buf = BytesMut::with_capacity(20);
        buf.put_u16(LEAVE);
        buf.put_u128(self.uuid.as_u128());
        buf.put_u8(flags);
        buf.put_u8(0u8);
        buf
    }
}

impl Default for Node226 {
    fn default() -> Node226 {
        Node226::new(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(226, 226, 226, 226)),
            22626,
        ))
    }
}

impl Display for Node226 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}@{}",
            self.uuid
                .to_hyphenated()
                .encode_upper(&mut Uuid::encode_buffer()),
            self.group_addr
        )
    }
}
pub fn start_node(
    node226: Node226,
    handler: NodeHandler<Signals226>,
    listener: NodeListener<Signals226>,
) {
    let multicast_addr = node226.group_addr.to_string();
    info!("Joining net226 multicast group on {} ", &multicast_addr);
    let (mc_endpoint, mc_socket) = handler
        .network()
        .connect(Transport::Udp, multicast_addr)
        .unwrap();
    info!("Listening on net226 multicast at {} ", &mc_socket);

    info!("Starting tcp listener");
    let (_, tcp_addr) = handler
        .network()
        .listen(Transport::Tcp, "0.0.0.0:0")
        .unwrap();
    info!(
        "Node {} will listen on tcp at port {}",
        node226,
        tcp_addr.port(),
    );
    let mut last_msg_time = SystemTime::now();

    listener.for_each(move |event| match event {
        NodeEvent::Signal(signal) => {
            handle_signals(
                signal,
                &mut last_msg_time,
                tcp_addr.port(),
                node226,
                &handler,
                mc_endpoint,
            );
        }
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Connected(conn_endpoint, _connected) => {
                info!("Message from {}", conn_endpoint.addr().ip());
                if conn_endpoint == mc_endpoint {
                    info!("Notifying the network about my presence");
                    handler.network().send(
                        mc_endpoint,
                        &node226.join_msg(tcp_addr.port(), 0b00000001)[..],
                    );
                    last_msg_time = SystemTime::now();
                    handler
                        .network()
                        .listen(Transport::Udp, node226.group_addr.to_string())
                        .unwrap();
                    info!("Node226 {} ready to for action ", &node226);

                    handler
                        .signals()
                        .send_with_timer(Signals226::Heartbeat, node226.heart_beat);
                }
            }

            NetEvent::Accepted(neighbour_endpoint, res_id) => {
                info!(
                    "Got a new tcp connection request from {} {}",
                    neighbour_endpoint, res_id
                );
            }
            NetEvent::Message(conn_endpoint, data) => {
                debug!("Processing message on {} ", conn_endpoint);
                if conn_endpoint.addr() == mc_socket {
                    debug!(
                        "Multicast message on {} {}",
                        conn_endpoint.resource_id(),
                        conn_endpoint.addr()
                    );
                    let mut fr_iter = FrameIterator::new(data);
                    for frame in &mut fr_iter {
                        info!("Got Multicast Message {}", frame);
                    }
                } else {
                }
            }
            NetEvent::Disconnected(conn_endpoint) => {
                if conn_endpoint.addr() == mc_socket {
                    //maybe we shall do a restart sequence ???
                    info!("We are disconnected from the network226")
                }
            }
        },
    });
}

#[inline]
fn handle_signals(
    signal: Signals226,
    last_msg_time: &mut SystemTime,
    port: u16,
    node226: Node226,
    handler: &NodeHandler<Signals226>,
    mc_endpoint: message_io::network::Endpoint,
) {
    trace!("Got signal {:?}", signal);
    match signal {
        Signals226::Heartbeat => {
            let now = SystemTime::now();
            if now.duration_since(*last_msg_time).unwrap_or_default() > node226.heart_beat {
                debug!("Heart beating");
                handler
                    .network()
                    .send(mc_endpoint, &node226.hb_msg(port, 0b00000001)[..]);
                *last_msg_time = now;
                handler
                    .signals()
                    .send_with_timer(Signals226::Heartbeat, node226.heart_beat);
            }
        }
        Signals226::Quit => {
            info!("Got Quit signal. Sending bye message");
            handler
                .network()
                .send(mc_endpoint, &node226.leave_msg(0b00000001)[..]);
            handler
                .signals()
                .send_with_timer(Signals226::ShutdownNow, Duration::from_millis(333));
        }
        Signals226::ShutdownNow => {
            info!("Shutingdown...");
            handler.stop();
        }
    }
}
