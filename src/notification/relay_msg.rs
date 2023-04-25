use crate::impl_from_variant_unwrap;
use crate::{Enr, MessageNonce, Notification, REALYMSG_MSG_TYPE};
use rlp::RlpStream;
use std::{
    fmt,
    fmt::{Debug, Display},
};

/// Nonce of the timed out FINDNODE request that triggered the initiation of this hole punching
/// attempt.
type NonceOfTimedOutMessage = MessageNonce;

/// A relayed hole punch notification sent to the target. Contains the enr of the initiator of the
/// hole punch and the nonce of the initiator's request that timed out.
///
/// The hole punch target uses the nonce to respond with WHOAREYOU to the initiator.
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

impl Display for RelayMsg {
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
