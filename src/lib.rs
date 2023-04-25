use async_trait::async_trait;
use rand::Rng;
use std::{
    fmt::{Debug, Display},
    net::{IpAddr, SocketAddr, UdpSocket},
    ops::RangeInclusive,
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
/// The default number of ports to try before concluding that the local node is behind NAT.
pub const DEFAULT_PORT_BIND_TRIES: usize = 4;
/// Port range that is not impossible to bind to.
pub const USER_AND_DYNAMIC_PORTS: RangeInclusive<u16> = 1025..=u16::MAX;

#[async_trait]
pub trait NatHolePunch {
    /// A type in discv5 for indexing sessions. Discv5 indexes sessions based on combination
    /// `(socket, node-id)`.
    type SessionIndex: Send + Sync;
    /// A discv5 error type.
    type Discv5Error: Display + Debug;
    /// A request times out. Should trigger the initiation of a hole punch attempt, given a
    /// transitive route to the target exists.
    async fn on_request_time_out(
        &mut self,
        relay: Self::SessionIndex,
        local_enr: Enr, // initiator-enr
        timed_out_message_nonce: MessageNonce,
        target_session_index: Self::SessionIndex,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// A notification is received over discv5.
    async fn on_notification(
        &mut self,
        decrypted_notif: &[u8],
    ) -> Result<(), HolePunchError<Self::Discv5Error>> {
        match Notification::rlp_decode(decrypted_notif)? {
            Notification::RelayInit(relay_init_notif) => self.on_relay_init(relay_init_notif).await,
            Notification::RelayMsg(relay_msg_notif) => self.on_relay_msg(relay_msg_notif).await,
        }
    }
    /// A [`RelayInit`] notification is received indicating this node is the relay. Should trigger
    /// sending a [`RelayMsg`] to the target.
    async fn on_relay_init(
        &mut self,
        notif: RelayInit,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// A [`RelayMsg`] notification is received indicating this node is the target. Should trigger
    /// a WHOAREYOU to be sent to the initiator using the `nonce` in the [`RelayMsg`].
    async fn on_relay_msg(
        &mut self,
        notif: RelayMsg,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
    /// A punched hole closes. Should trigger an empty packet to be sent to the peer.
    async fn on_hole_punch_expired(
        &mut self,
        dst: SocketAddr,
    ) -> Result<(), HolePunchError<Self::Discv5Error>>;
}

/// Helper function to test if the local node is behind NAT based on the node's observed reachable
/// socket.
pub fn is_behind_nat(observed_ip: IpAddr, unused_port_range: Option<RangeInclusive<u16>>) -> bool {
    // If the node cannot bind to the observed address at any of some random ports, we
    // conclude it is behind NAT.
    let mut rng = rand::thread_rng();
    let unused_port_range = match unused_port_range {
        Some(range) => range,
        None => USER_AND_DYNAMIC_PORTS,
    };
    for _ in 0..DEFAULT_PORT_BIND_TRIES {
        let rnd_port: u16 = rng.gen_range(unused_port_range.clone());
        let socket_addr: SocketAddr = format!("{}:{}", observed_ip, rnd_port).parse().unwrap();
        if UdpSocket::bind(socket_addr).is_ok() {
            return false;
        }
    }
    true
}
