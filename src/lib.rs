use async_trait::async_trait;
use rlp::DecoderError;

mod node_address;
mod notification;

pub use node_address::NodeAddress;
pub use notification::{MessageNonce, Notification, RelayInit, RelayMsg};

#[macro_export]
macro_rules! impl_from_variant_wrap {
    ($from_type: ty, $to_type: ty, $variant: path) => {
        impl From<$from_type> for $to_type {
            fn from(e: $from_type) -> Self {
                $variant(e)
            }
        }
    };
}
#[macro_export]
macro_rules! impl_from_variant_unwrap {
    ($from_type: ty, $to_type: ty, $variant: path) => {
        impl From<$from_type> for $to_type {
            fn from(e: $from_type) -> Self {
                if let $variant(v) = e {
                    return v;
                }
                panic!("Bad impl of From")
            }
        }
    };
}

#[derive(Debug)]
pub enum HolePunchError {
    NotificationError(DecoderError),
    Session(&'static str),
    RelayError(&'static str),
    TargetError(&'static str),
}

impl_from_variant_wrap!(DecoderError, HolePunchError, Self::NotificationError);

#[async_trait]
pub trait NatHolePunch {
    /// A FINDNODE request, as part of a find node query, has timed out. Hole punching is
    /// initiated. The node which passed the hole punch target peer in a NODES response to us is
    /// used as relay.
    async fn on_time_out(
        &mut self,
        relay: NodeAddress,
        local_node_address: NodeAddress,
        message_nonce: MessageNonce,
        target_node_address: NodeAddress,
    ) -> Result<(), HolePunchError>;
    /// Handle a notification packet received over discv5 used for hole punching.
    async fn on_notification(
        &mut self,
        notif_sender: NodeAddress,
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<(), HolePunchError> {
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
        session_index: NodeAddress, // notif sender
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<Vec<u8>, HolePunchError>;
    /// This node receives a message to relay. It should send a [`RelayMsg`] to the `target` in
    /// the [`RelayInit`] notification.
    async fn on_relay_init(&mut self, notif: RelayInit) -> Result<(), HolePunchError>;
    /// This node received a relayed message and should punch a hole in its NAT for the initiator.
    async fn on_relay_msg(&mut self, notif: RelayMsg) -> Result<(), HolePunchError>;
}
