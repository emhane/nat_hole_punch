use async_trait::async_trait;
use std::fmt::{Debug, Display};

mod error;
mod macro_rules;
mod notification;

pub use error::HolePunchError;
pub use notification::{
    MessageNonce, NodeId, Notification, RelayInit, RelayMsg, MESSAGE_NONCE_LENGTH, NODE_ID_LENGTH,
    REALYINIT_MSG_TYPE, REALYMSG_MSG_TYPE,
};

/// The expected shortest lifetime in most NAT configurations of a punched hole in seconds.
pub const DEFAULT_HOLE_PUNCH_LIFETIME: u64 = 20;

#[async_trait]
pub trait NatHolePunch {
    /// A standardised type for sending a node address over discv5.
    type TEnr: rlp::Encodable + rlp::Decodable + Send + Sync + Display + Debug + PartialEq + Eq;
    /// A type for indexing sessions. Each `(node-id, socket-address)` combination gets a unique
    /// session in discv5.
    type TNodeAddress: Send + Sync;
    /// Discv5 error.
    type TDiscv5Error: Display + Debug;
    /// A FINDNODE request, as part of a find node query, has timed out. Hole punching is
    /// initiated. The node which passed the hole punch target peer in a NODES response to us is
    /// used as relay.
    async fn on_time_out(
        &mut self,
        relay: Self::TNodeAddress,
        local_enr: Self::TEnr, // initiator-enr
        timed_out_message_nonce: MessageNonce,
        target_session_index: Self::TNodeAddress,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>>;
    /// Handle a notification packet received over discv5 used for hole punching. Decrypt a
    /// notification with session keys held for the notification sender, just like for a
    /// discv5 message. Notifications should differentiate themselves from discv5 messages
    /// (request or response) in the way they handle a session, or rather the absence of a
    /// session. Notifications that can't be decrypted with existing session keys should be
    /// dropped.
    async fn on_notification(
        &mut self,
        decrypted_notif: &Vec<u8>,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>> {
        match Notification::rlp_decode(&decrypted_notif)? {
            Notification::RelayInit(relay_init_notif) => self.on_relay_init(relay_init_notif).await,
            Notification::RelayMsg(relay_msg_notif) => self.on_relay_msg(relay_msg_notif).await,
        }
    }
    /// This node receives a message to relay. It should send a [`RelayMsg`] to the `target` in
    /// the [`RelayInit`] notification.
    async fn on_relay_init(
        &mut self,
        notif: RelayInit<Self::TEnr>,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>>;
    /// This node received a relayed message and should punch a hole in its NAT for the initiator
    /// by sending a WHOAREYOU packet wrapping the nonce in the [`RelayMsg`].
    async fn on_relay_msg(
        &mut self,
        notif: RelayMsg<Self::TEnr>,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>>;
    /// If no packet is sent to a peer within [`DEFAULT_HOLE_PUNCH_LIFETIME`], that hole will
    /// close. An empty packet should be sent to the peer to keep the hole punched. An empty
    /// packet spares the sender the work of encryption, as any hardcoded bytes would have to be
    /// masked to circumvent packet filtering.
    async fn on_hole_punch_expired(
        &mut self,
        dst: Self::TNodeAddress,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>>;
}

/// Checks if this packet is empty indicating it is probably a packet to keep a hole punched.
pub fn is_keep_hole_punched_packet(bytes_read: usize) -> bool {
    bytes_read == 0
}
