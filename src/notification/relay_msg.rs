use crate::impl_from_variant_unwrap;
use crate::{Enr, MessageNonce, Notification, REALYMSG_MSG_TYPE};
use rlp::RlpStream;
use std::fmt;

/// Nonce of request that triggered the initiation of this hole punching attempt.
type NonceOfTimedOutMessage = MessageNonce;

/// A notification sent from the initiator to the relay. Contains the enr of the initiator and the
/// nonce of the timed out request.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RelayMsg(pub Enr, pub NonceOfTimedOutMessage);

impl_from_variant_unwrap!(, Notification, RelayMsg, Notification::RelayMsg);

impl RelayMsg {
    pub fn rlp_encode(self) -> Vec<u8> {
        let RelayMsg(initiator, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(2);
        s.append(&initiator);
        s.append(&(&nonce as &[u8]));

        let mut buf: Vec<u8> = Vec::with_capacity(312);
        buf.push(REALYMSG_MSG_TYPE);
        buf.extend_from_slice(&s.out());
        buf
    }
}

impl fmt::Display for RelayMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let initiator = &self.0;
        let nonce = hex::encode(self.1);
        write!(
            f,
            "RelayMsg: Initiator: {}, Nonce: 0x{}..{}",
            initiator,
            &nonce[0..2],
            &nonce[nonce.len() - 2..]
        )
    }
}
