use crate::impl_from_variant_unwrap;
use crate::{Enr, MessageNonce, NodeId, Notification, REALYINIT_MSG_TYPE};
use rlp::RlpStream;
use std::fmt;

/// Nonce of request that triggered the initiation of this hole punching attempt.
type NonceOfTimedOutMessage = MessageNonce;

/// A notification sent from the initiator to the relay. Contains the enr of the initiator, the
/// nonce of the timed out request and the node id of the target.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RelayInit(pub Enr, pub NodeId, pub NonceOfTimedOutMessage);

impl_from_variant_unwrap!(, Notification, RelayInit, Notification::RelayInit);

impl RelayInit {
    pub fn rlp_encode(self) -> Vec<u8> {
        let RelayInit(initiator, target, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(3);
        s.append(&initiator);
        s.append(&(&target as &[u8]));
        s.append(&(&nonce as &[u8]));

        let mut buf: Vec<u8> = Vec::with_capacity(280);
        buf.push(REALYINIT_MSG_TYPE);
        buf.extend_from_slice(&s.out());
        buf
    }
}

impl fmt::Display for RelayInit {
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
