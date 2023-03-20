use async_trait::async_trait;
use enr::NodeId;
use rlp::DecoderError;
use std::net::SocketAddr;

/// Discv5 message nonce length in bytes.
pub const MESSAGE_NONCE_LENGTH: usize = 12;
/// Discv5 timed out FINDNODE request's packet nonce.
pub type MessageNonce = [u8; MESSAGE_NONCE_LENGTH];
pub type NonceOfTimedOutMessage = MessageNonce;

macro_rules! impl_from_variant_wrap {
    ($from_type: ty, $to_type: ty, $variant: path) => {
        impl From<$from_type> for $to_type {
            fn from(e: $from_type) -> Self {
                $variant(e)
            }
        }
    };
}

#[derive(Debug)]
pub enum HolePunchError {
    NotificationError(DecoderError),
    Session(String),
    RelayError(String),
    TargetError(String),
}

#[async_trait]
pub trait NatHolePunch {
    /// A FINDNODE request, as part of a find node query, has timed out. Hole punching is
    /// initiated. The node which passed the hole punch target peer in a NODES response to us is
    /// used as relay.
    async fn on_time_out(
        &mut self,
        relay: NodeAddress,
        notif: RelayInit,
    ) -> Result<(), HolePunchError>;
    /// Handle a notification received over discv5 used for hole punching.
    async fn on_notification_packet(
        &mut self,
        notif_sender: NodeAddress,
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<(), HolePunchError> {
        let decrypted_notif = self
            .decrypt_notification(notif_sender, notif_nonce, notif, authenticated_data)
            .await?;

        match Notification::decode(decrypted_notif)? {
            Notification::RelayInit(relay_init_notif) => self.on_relay_init(relay_init_notif).await,
            Notification::RelayMsg(relay_msg_notif) => self.on_relay_msg(relay_msg_notif).await,
        }
    }
    /// Decrypt a notification with the session keys held for the sender, just like for a discv5
    /// message. Notifications differentiate themsleves from discv5 messages (request or response)
    /// in the way they handle a session, or rather the absence of a session. The duration of a
    /// roundtrip in a hole punch relay circuit is bound by the duration of an average router's
    /// time out for a udp entry for a given connection. Notifications that can't be decrypted
    /// should be dropped to stay within bounds.
    async fn decrypt_notification(
        &mut self,
        notif_sender: NodeAddress,
        notif_nonce: MessageNonce,
        notif: &[u8],
        authenticated_data: &[u8],
    ) -> Result<Vec<u8>, HolePunchError>;
    /// This node receives a message to relay.
    async fn on_relay_init(&mut self, notif: RelayInit) -> Result<(), HolePunchError>;
    /// This node received a relayed message and should punch a hole in its NAT for the initiator.
    async fn on_relay_msg(&mut self, notif: RelayMsg) -> Result<(), HolePunchError>;
}

/// A unicast notification sent over discv5.
pub enum Notification {
    /// Initialise a one-shot relay circuit.
    RelayInit(RelayInit),
    /// A relayed notification.
    RelayMsg(RelayMsg),
}

/// A hole punch notification sent to the relay. Contains the node address of the initiator of the
/// hole punch, the nonce of the request from the initiator to the target that triggered
/// `on_time_out` and the node address of the hole punch target peer.
pub struct RelayInit(NodeAddress, NonceOfTimedOutMessage, NodeAddress);
/// A relayed hole punch notification sent to the target. Contains the node address of the
/// initiator of the hole punch and the nonce of the initiator's request that timed out, so the
/// hole punch target peer can respond with WHOAREYOU to the initiator.
pub struct RelayMsg(NodeAddress, NonceOfTimedOutMessage);

impl_from_variant_wrap!(RelayInit, Notification, Self::RelayInit);
impl_from_variant_wrap!(RelayMsg, Notification, Self::RelayMsg);

/// Discv5 node address.
pub struct NodeAddress {
    pub socket_addr: SocketAddr,
    pub node_id: NodeId,
}

impl Notification {
    fn decode(message: Vec<u8>) -> Result<Self, HolePunchError> {
        // check flag todo(emhane)
        todo!()
    }
    fn encode(self) -> Result<Vec<u8>, HolePunchError> {
        /// Encodes a Message to RLP-encoded bytes.
        let mut buf = Vec::with_capacity(10);
        /*let msg_type = self.msg_type();
            buf.push(msg_type);
            let id = &self.id;
            match self.body {
                RequestBody::Ping { enr_seq } => {
                    let mut s = RlpStream::new();
                    s.begin_list(2);
                    s.append(&id.as_bytes());
                    s.append(&enr_seq);
                    buf.extend_from_slice(&s.out());
                    buf
                }
                RequestBody::FindNode { distances } => {
                    let mut s = RlpStream::new();
                    s.begin_list(2);
                    s.append(&id.as_bytes());
                    s.begin_list(distances.len());
                    for distance in distances {
                        s.append(&distance);
                    }
                    buf.extend_from_slice(&s.out());
                    buf
                }
        }*/
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enocde_decode_relay_init() {}

    #[test]
    fn test_enocde_decode_relay_msg() {}
}
