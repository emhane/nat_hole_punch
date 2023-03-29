use async_trait::async_trait;
use std::fmt::{Debug, Display};

mod error;
mod notification;

pub use error::HolePunchError;
pub use notification::{MessageNonce, NodeId, Notification, RelayInit, RelayMsg};

#[async_trait]
pub trait NatHolePunch {
    /// A standardised type for sending a node address over discv5.
    type TEnr: rlp::Encodable + rlp::Decodable + Send + Sync;
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
        local_enr: Self::TEnr,
        timed_out_message_nonce: MessageNonce,
        target_session_index: Self::TNodeAddress,
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>>;
    /// Handle a notification packet received over discv5 used for hole punching.
    async fn on_notification(
        &mut self,
        notif_sender: Self::TNodeAddress,
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<(), HolePunchError<Self::TDiscv5Error>> {
        let decrypted_notif = self
            .handle_decryption_with_session(notif_sender, notif_nonce, notif, authenticated_data)
            .await?;

        match Notification::rlp_decode(&decrypted_notif)? {
            Notification::RelayInit(relay_init_notif) => self.on_relay_init(relay_init_notif).await,
            Notification::RelayMsg(relay_msg_notif) => self.on_relay_msg(relay_msg_notif).await,
        }
    }
    /// Decrypt a notification with session keys held for the notification sender, just like for a
    /// discv5 message. Notifications should differentiate themsleves from discv5 messages
    /// (request or response) in the way they handle a session, or rather the absence of a
    /// session. Notifications that can't be decrypted with existing session keys should be
    /// dropped.
    async fn handle_decryption_with_session(
        &mut self,
        session_index: Self::TNodeAddress, // notif sender
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<Vec<u8>, HolePunchError<Self::TDiscv5Error>>;
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
}
