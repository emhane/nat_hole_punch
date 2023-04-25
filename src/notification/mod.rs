use crate::impl_from_variant_wrap;
use enr::CombinedKey;
use rlp::{DecoderError, Rlp};
use std::{
    fmt,
    fmt::{Debug, Display},
};

mod relay_init;
mod relay_msg;

pub use relay_init::RelayInit;
pub use relay_msg::RelayMsg;

/// Discv5 message nonce length in bytes.
pub const MESSAGE_NONCE_LENGTH: usize = 12;
/// Discv5 node id length in bytes.
pub const NODE_ID_LENGTH: usize = 32;
/// Notification types as according to wire protocol.
///
/// RelayInit notification type.
pub const REALYINIT_MSG_TYPE: u8 = 7;
/// RelayMsg notification type.
pub const REALYMSG_MSG_TYPE: u8 = 8;

/// Enr using same key type as rust discv5.
pub type Enr = enr::Enr<CombinedKey>;
/// Discv5 message nonce.
pub type MessageNonce = [u8; MESSAGE_NONCE_LENGTH];
/// Discv5 node id.
pub type NodeId = [u8; NODE_ID_LENGTH];

/// A unicast notification sent over discv5.
#[derive(Debug, PartialEq, Eq)]
pub enum Notification {
    /// Initialise a one-shot relay circuit for hole punching.
    RelayInit(RelayInit),
    /// A relayed notification for hole punching.
    RelayMsg(RelayMsg),
}

impl_from_variant_wrap!(, RelayInit, Notification, Self::RelayInit);
impl_from_variant_wrap!(, RelayMsg, Notification, Self::RelayMsg);

impl Notification {
    pub fn rlp_decode(data: &[u8]) -> Result<Self, DecoderError> {
        if data.len() < 3 {
            return Err(DecoderError::RlpIsTooShort);
        }
        let msg_type = data[0];

        let rlp = Rlp::new(&data[1..]);
        let list_len = rlp.item_count()?;
        if list_len < 2 {
            return Err(DecoderError::RlpIsTooShort);
        }

        let initiator = rlp.val_at::<Enr>(0)?;

        let nonce_bytes = rlp.val_at::<Vec<u8>>(list_len - 1)?;

        if nonce_bytes.len() > MESSAGE_NONCE_LENGTH {
            return Err(DecoderError::RlpIsTooBig);
        }
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        match msg_type {
            REALYINIT_MSG_TYPE => {
                if list_len != 3 {
                    return Err(DecoderError::RlpIncorrectListLen);
                }

                let tgt_bytes = rlp.val_at::<Vec<u8>>(1)?;
                if tgt_bytes.len() > NODE_ID_LENGTH {
                    println!("waa");
                    return Err(DecoderError::RlpIsTooBig);
                }
                let mut tgt = [0u8; NODE_ID_LENGTH];
                tgt[NODE_ID_LENGTH - tgt_bytes.len()..].copy_from_slice(&tgt_bytes);

                Ok(RelayInit(initiator, tgt, nonce).into())
            }
            REALYMSG_MSG_TYPE => {
                if list_len != 2 {
                    return Err(DecoderError::RlpIncorrectListLen);
                }
                Ok(RelayMsg(initiator, nonce).into())
            }
            _ => Err(DecoderError::Custom("invalid notification type")),
        }
    }
}

impl Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Notification::RelayInit(notif) => write!(f, "Notification: {}", notif),
            Notification::RelayMsg(notif) => write!(f, "Notification: {}", notif),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enr::{CombinedKey, EnrBuilder};

    #[test]
    fn test_enocde_decode_relay_init() {
        // generate a new enr key for the initiator
        let enr_key = CombinedKey::generate_secp256k1();
        // construct the initiator's ENR
        let inr_enr = EnrBuilder::new("v4").build(&enr_key).unwrap();

        // generate a new enr key for the target
        let enr_key_tgt = CombinedKey::generate_secp256k1();
        // construct the target's ENR
        let tgt_enr = EnrBuilder::new("v4").build(&enr_key_tgt).unwrap();
        let tgt_node_id = tgt_enr.node_id().raw();
        println!("{:?}", tgt_node_id);
        let nonce_bytes = hex::decode("47644922f5d6e951051051ac").unwrap();
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        println!("{:?}", nonce_bytes);
        let notif = RelayInit(inr_enr, tgt_node_id, nonce);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }

    #[test]
    fn test_enocde_decode_relay_msg() {
        // generate a new enr key for the initiator
        let enr_key = CombinedKey::generate_secp256k1();
        // construct the initiator's ENR
        let inr_enr = EnrBuilder::new("v4").build(&enr_key).unwrap();

        let nonce_bytes = hex::decode("9951051051aceb").unwrap();
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        let notif = RelayMsg(inr_enr, nonce);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }
}
