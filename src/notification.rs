use crate::{impl_from_variant_unwrap, impl_from_variant_wrap};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

/// Notification types for rlp encoding notifications.
pub const REALY_INIT_NOTIF_TYPE: u8 = 0;
pub const REALY_MSG_NOTIF_TYPE: u8 = 1;
/// Discv5 message nonce length in bytes.
pub const MESSAGE_NONCE_LENGTH: usize = 12;
/// Discv5 node id length in bytes.
pub const NODE_ID_LENGTH: usize = 32;

/// Discv5 message nonce.
pub type MessageNonce = [u8; MESSAGE_NONCE_LENGTH];
/// The message nonce of the timed out FINDNODE request that triggered the initiation of this hole
/// punching attempt.
pub type NonceOfTimedOutMessage = MessageNonce;

/// Discv5 node id.
pub type NodeId = [u8; NODE_ID_LENGTH];

/// A unicast notification sent over discv5.
#[derive(Debug)]
pub enum Notification<TEnr: Encodable + Decodable> {
    /// Initialise a one-shot relay circuit for hole punching.
    RelayInit(RelayInit<TEnr>),
    /// A relayed notification for hole punching.
    RelayMsg(RelayMsg<TEnr>),
}

/// A hole punch notification sent to the relay. Contains the enr of the initiator of the hole
/// punch, the nonce of the request from the initiator to the target that triggered `on_time_out`
/// and the node id of the hole punch target peer.
#[derive(Clone, PartialEq, Debug)]
pub struct RelayInit<TEnr: Encodable + Decodable>(pub TEnr, pub NodeId, pub NonceOfTimedOutMessage);
/// A relayed hole punch notification sent to the target. Contains the enr of the initiator of the
/// hole punch and the nonce of the initiator's request that timed out, so the hole punch target
/// peer can respond with WHOAREYOU to the initiator.
#[derive(Clone, PartialEq, Debug)]
pub struct RelayMsg<TEnr: Encodable + Decodable>(pub TEnr, pub NonceOfTimedOutMessage);

impl_from_variant_wrap!(<TEnr: Encodable + Decodable,>, RelayInit<TEnr>, Notification<TEnr>, Self::RelayInit);
impl_from_variant_unwrap!(<TEnr: Encodable + Decodable,>, Notification<TEnr>, RelayInit<TEnr>, Notification::RelayInit);
impl_from_variant_wrap!(<TEnr: Encodable + Decodable,>, RelayMsg<TEnr>, Notification<TEnr>, Self::RelayMsg);
impl_from_variant_unwrap!(<TEnr: Encodable + Decodable,>, Notification<TEnr>, RelayMsg<TEnr>, Notification::RelayMsg);

impl<TEnr> Notification<TEnr>
where
    TEnr: Encodable + Decodable,
{
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

        let initiator = rlp.val_at::<TEnr>(0)?;

        let nonce_bytes = rlp.val_at::<Vec<u8>>(list_len - 1)?;

        if nonce_bytes.len() > MESSAGE_NONCE_LENGTH {
            return Err(DecoderError::RlpIsTooBig);
        }
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        match msg_type {
            REALY_INIT_NOTIF_TYPE => {
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
            REALY_MSG_NOTIF_TYPE => {
                if list_len != 2 {
                    return Err(DecoderError::RlpIncorrectListLen);
                }
                Ok(RelayMsg(initiator, nonce).into())
            }
            _ => Err(DecoderError::Custom("invalid notification type")),
        }
    }
}

impl<TEnr> RelayInit<TEnr>
where
    TEnr: Encodable + Decodable,
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

impl<TEnr> RelayMsg<TEnr>
where
    TEnr: Encodable + Decodable,
{
    pub fn rlp_encode(self) -> Vec<u8> {
        let RelayMsg(initiator, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(2);
        s.append(&initiator);
        s.append(&(&nonce as &[u8]));

        let mut buf: Vec<u8> = Vec::with_capacity(312);
        buf.push(REALY_MSG_NOTIF_TYPE);
        buf.extend_from_slice(&s.out());
        buf
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
        let init_enr = EnrBuilder::new("v4").build(&enr_key).unwrap();

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
        let notif = RelayInit(init_enr, tgt_node_id, nonce);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }

    #[test]
    fn test_enocde_decode_relay_msg() {
        // generate a new enr key for the initiator
        let enr_key = CombinedKey::generate_secp256k1();
        // construct the initiator's ENR
        let init_enr = EnrBuilder::new("v4").build(&enr_key).unwrap();

        let nonce_bytes = hex::decode("9951051051aceb").unwrap();
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        let notif = RelayMsg(init_enr, nonce);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }
}
