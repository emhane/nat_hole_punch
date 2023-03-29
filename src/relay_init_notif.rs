use crate::impl_from_variant_unwrap;
use crate::{MessageNonce, NodeId, Notification, REALY_INIT_NOTIF_TYPE};
use rlp::{Decodable, Encodable, RlpStream};
use std::{fmt, fmt::Display};

/// Nonce of the timed out FINDNODE request that triggered the initiation of this hole punching
/// attempt.
type NonceOfTimedOutMessage = MessageNonce;

/// A hole punch notification sent to the relay. Contains the enr of the initiator of the hole
/// punch (the sender), the nonce of the request from the initiator to the target that triggered
/// `on_time_out` and the node id of the hole punch target peer.
#[derive(Clone, PartialEq, Debug)]
pub struct RelayInit<TEnr: Encodable + Decodable + Display>(
    pub TEnr,
    pub NodeId,
    pub NonceOfTimedOutMessage,
);

impl_from_variant_unwrap!(<TEnr: Encodable + Decodable + Display,>, Notification<TEnr>, RelayInit<TEnr>, Notification::RelayInit);

impl<TEnr> RelayInit<TEnr>
where
    TEnr: Encodable + Decodable + Display,
{
    pub fn rlp_encode(self) -> Vec<u8> {
        let RelayInit(initiator, target, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(3);
        s.append(&initiator);
        s.append(&(&target as &[u8]));
        s.append(&(&nonce as &[u8]));

        let mut buf: Vec<u8> = Vec::with_capacity(280);
        buf.push(REALY_INIT_NOTIF_TYPE);
        buf.extend_from_slice(&s.out());
        buf
    }
}

impl<TEnr: Encodable + Decodable + Display> Display for RelayInit<TEnr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let initiator = &self.0;
        let tgt = hex::encode(self.1);
        let nonce = hex::encode(self.2);
        write!(
            f,
            "RelayInit: Initiator: {}, Target: 0x{}..{}, Nonce: 0x{}..{}",
            initiator,
            &tgt[0..4],
            &tgt[tgt.len() - 4..],
            &nonce[0..2],
            &nonce[nonce.len() - 2..]
        )
    }
}
