use async_trait::async_trait;
use rand::Rng;
use std::{
    fmt::{Debug, Display},
    net::{IpAddr, SocketAddr, UdpSocket},
};

mod error;
mod macro_rules;
mod notification;

pub use error::HolePunchError;
pub use notification::{
    Enr, MessageNonce, NodeId, Notification, RelayInit, RelayMsg, MESSAGE_NONCE_LENGTH,
    NODE_ID_LENGTH, REALYINIT_MSG_TYPE, REALYMSG_MSG_TYPE,
};

/// The expected shortest lifetime in most NAT configurations of a punched hole in seconds.
pub const DEFAULT_HOLE_PUNCH_LIFETIME: u64 = 20;
/// The number of ports to try before concluding that a node is behind a NAT.
pub const DEFAULT_PORT_BIND_TRIES: usize = 4;
/// The min port to try binding to in order to test what address realm the node is in.
pub const DEFAULT_MIN_PORT: u16 = 1025;
/// The max port to try binding to in order to test what address realm the node is in.
pub const DEFAULT_MAX_PORT: u16 = u16::MAX;

#[async_trait]
pub trait NatHolePunch {
    /// A type in discv5 for indexing sessions.
    type SessionIndex: Send + Sync;
    /// A discv5 error type.
    type Discv5Error: Display + Debug;
    /// A FINDNODE request, as part of a find node query, has timed out. Hole punching is
    /// initiated. The node which passed the hole punch target peer in a NODES response to us is
    /// used as relay.
    async fn on_time_out(
        &mut self,
        relay: Self::SessionIndex,
        local_enr: Enr, // initiator-enr
        timed_out_message_nonce: MessageNonce,
        target_session_index: Self::SessionIndex,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// Handle a notification message received over discv5 used for hole punching.
    async fn on_notification(
        &mut self,
        decrypted_notif: &[u8],
    ) -> Result<(), HolePunchError<Self::Discv5Error>> {
        match Notification::rlp_decode(decrypted_notif)? {
            Notification::RelayInit(relay_init_notif) => self.on_relay_init(relay_init_notif).await,
            Notification::RelayMsg(relay_msg_notif) => self.on_relay_msg(relay_msg_notif).await,
        }
    }
    /// This node receives a message to relay. It should send a [`RelayMsg`] to the `target` in
    /// the [`RelayInit`] notification.
    async fn on_relay_init(
        &mut self,
        notif: RelayInit,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// This node received a relayed message and should punch a hole in its NAT for the initiator
    /// by sending a WHOAREYOU packet wrapping the nonce in the [`RelayMsg`].
    async fn on_relay_msg(
        &mut self,
        notif: RelayMsg,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// If no packet is sent to a peer within [`DEFAULT_HOLE_PUNCH_LIFETIME`], that hole will
    /// close. An empty packet should be sent to the peer to keep the hole punched. An empty
    /// packet spares the sender the work of encryption, as any hardcoded bytes would have to be
    /// masked to circumvent packet filtering.
    async fn on_hole_punch_expired(
        &mut self,
        dst: SocketAddr,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
}

/// Helper function to check if this packet is empty indicating it is probably a packet to keep a
/// hole punched.
pub fn is_keep_hole_punched_packet(bytes_read: usize) -> bool {
    bytes_read == 0
}

/// Helper function to test if the local node is behind NAT based on the node's observed
/// socket as configured on start-up as the advertised socket, or as reported by peers at
/// runtime. If the node is not behind NAT, it is most likely that the program can bind to the
/// observed IP address at some port out of a random subset of ports from a range of probably
/// unused ports, defaulting to the port range 1025-65536.
pub fn is_behind_nat(
    observed_ip: IpAddr,
    (min_unused_port, max_unused_port): (Option<u16>, Option<u16>),
) -> bool {
    // If the node cannot bind to the observed address at any of some random ports, we
    // conclude it is behind NAT.
    let mut rng = rand::thread_rng();
    let min_port = match min_unused_port {
        Some(port) => port,
        None => DEFAULT_MIN_PORT,
    };
    let max_port = match max_unused_port {
        Some(port) => port,
        None => DEFAULT_MAX_PORT,
    };
    for _ in 0..DEFAULT_PORT_BIND_TRIES {
        let rnd_port: u16 = rng.gen_range(min_port..=max_port);
        let socket_addr: SocketAddr = format!("{}:{}", observed_ip, rnd_port).parse().unwrap();
        if UdpSocket::bind(socket_addr).is_ok() {
            return false;
        }
    }
    true
}
