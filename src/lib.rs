use async_trait::async_trait;
use rlp::DecoderError;

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
    RelayError(String),
    TargetError(String),
}

// Pin<Box<dyn Future<Output = Result<(), HolePunchError>> + Send, Global>>
#[async_trait]
pub trait NatHolePunch {
    /// A FINDNODE request, as part of a find node query, has timed out. Hole punching is
    /// initiated using the node which passed the hole punch target peer in a NODES response to us
    /// as relay.
    async fn on_time_out(
        &mut self,
        relay: NodeAddress,
        notif: RelayInit,
    ) -> Result<(), HolePunchError>;
    /// Handle a notification received over discv5 used for hole punching.
    async fn on_notification(&mut self, message: Vec<u8>) -> Result<(), HolePunchError> {
        match Notification::decode(message)? {
            Notification::RelayInit(notif) => self.on_relay_init(notif).await,
            Notification::RelayMsg(notif) => self.on_relay_msg(notif).await,
        }
    }
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

/// The node address of the initiator of the hole punch, a nonce from the timed out request from
/// the initiator to the target that triggers the hole punch and the node address of the hole
/// punch target peer.
pub struct RelayInit(NodeAddress, [u8; 12], NodeAddress);
/// The node address of the initiator of the hole punch and a nonce from the initiator, so the
/// hole punch target peer can respond with WHOAREYOU to the initiator.
pub struct RelayMsg(NodeAddress, [u8; 12]);

impl_from_variant_wrap!(RelayInit, Notification, Self::RelayInit);
impl_from_variant_wrap!(RelayMsg, Notification, Self::RelayMsg);

impl Notification {
    fn decode(message: Vec<u8>) -> Result<Self, HolePunchError> {
        // check flag todo(emhane)
        todo!()
    }
    fn encode(self) -> Result<Vec<u8>, HolePunchError> {
        todo!()
    }
}

/// Discv5 node address.
pub struct NodeAddress {
    pub socket_addr: SocketAddr,
    pub node_id: NodeId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enocde_decode_relay_init() {}

    #[test]
    fn test_enocde_decode_relay_msg() {}
}
