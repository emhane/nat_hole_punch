use crate::{impl_from_variant_unwrap, impl_from_variant_wrap, HolePunchError, NodeAddress};
use rlp::{DecoderError, Rlp, RlpStream};

/// Notification types for rlp encoding notifications.
pub const REALY_INIT_NOTIF_TYPE: u8 = 0;
pub const REALY_MSG_NOTIF_TYPE: u8 = 1;

/// Discv5 message nonce length in bytes.
pub const MESSAGE_NONCE_LENGTH: usize = 12;
/// Discv5 message nonce.
pub type MessageNonce = [u8; MESSAGE_NONCE_LENGTH];
/// The message nonce of the timed out FINDNODE request that triggered the initiation of this hole
/// punching attempt.
pub type NonceOfTimedOutMessage = MessageNonce;

/// A unicast notification sent over discv5.
#[derive(Debug)]
pub enum Notification {
    /// Initialise a one-shot relay circuit.
    RelayInit(RelayInit),
    /// A relayed notification.
    RelayMsg(RelayMsg),
}

/// A hole punch notification sent to the relay. Contains the node address of the initiator of the
/// hole punch, the nonce of the request from the initiator to the target that triggered
/// `on_time_out` and the node address of the hole punch target peer.
#[derive(Clone, PartialEq, Debug)]
pub struct RelayInit(pub NodeAddress, pub NonceOfTimedOutMessage, pub NodeAddress);
/// A relayed hole punch notification sent to the target. Contains the node address of the
/// initiator of the hole punch and the nonce of the initiator's request that timed out, so the
/// hole punch target peer can respond with WHOAREYOU to the initiator.
#[derive(Clone, PartialEq, Debug)]
pub struct RelayMsg(pub NodeAddress, pub NonceOfTimedOutMessage);

impl_from_variant_wrap!(RelayInit, Notification, Self::RelayInit);
impl_from_variant_unwrap!(Notification, RelayInit, Notification::RelayInit);
impl_from_variant_wrap!(RelayMsg, Notification, Self::RelayMsg);
impl_from_variant_unwrap!(Notification, RelayMsg, Notification::RelayMsg);

impl Notification {
    pub fn rlp_decode(data: &[u8]) -> Result<Self, HolePunchError> {
        if data.len() < 3 {
            return Err(DecoderError::RlpIsTooShort.into());
        }
        let msg_type = data[0];

        let rlp = Rlp::new(&data[1..]);
        let list_len = rlp.item_count()?;
        println!("list len {}", list_len);
        if list_len < 2 {
            return Err(DecoderError::RlpIsTooShort.into());
        }
        let initiator = rlp.val_at::<NodeAddress>(0)?;
        let nonce_bytes = rlp.val_at::<Vec<u8>>(1)?;
        println!("list len {}", list_len);
        if nonce_bytes.len() > MESSAGE_NONCE_LENGTH {
            return Err(DecoderError::RlpIsTooBig.into());
        }
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);
        println!("list len {}", list_len);
        match msg_type {
            REALY_INIT_NOTIF_TYPE => {
                if list_len != 3 {
                    return Err(DecoderError::RlpIncorrectListLen.into());
                }
                let target = rlp.val_at::<NodeAddress>(2)?;
                Ok(RelayInit(initiator, nonce, target).into())
            }
            REALY_MSG_NOTIF_TYPE => {
                if list_len != 2 {
                    return Err(DecoderError::RlpIncorrectListLen.into());
                }
                Ok(RelayMsg(initiator, nonce).into())
            }
            _ => Err(DecoderError::Custom("invalid notification type").into()),
        }
    }
}

impl RelayInit {
    fn rlp_encode(self) -> Vec<u8> {
        let RelayInit(initiator, nonce, target) = self;

        let mut s = RlpStream::new();
        s.begin_list(3);
        s.append(&initiator);
        s.append(&(&nonce as &[u8]));
        s.append(&target);

        let mut buf = [0u8; 98];
        buf[0] = REALY_INIT_NOTIF_TYPE;
        buf[1..].copy_from_slice(&s.out());
        buf.to_vec()
    }
}

impl RelayMsg {
    fn rlp_encode(self) -> Vec<u8> {
        let RelayMsg(initiator, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(2);
        s.append(&initiator);
        s.append(&(&nonce as &[u8]));

        let mut buf = [0u8; 56];
        buf[0] = REALY_MSG_NOTIF_TYPE;
        buf[1..].copy_from_slice(&s.out());
        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enr::NodeId;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    #[test]
    fn test_enocde_decode_relay_init() {
        let ip: Ipv4Addr = "127.0.0.1".parse().unwrap();
        let port = 5000;
        let socket_addr: SocketAddr = SocketAddrV4::new(ip, port).into();
        let node_id = NodeId::parse(
            &hex::decode("fb757dc581730490a1d7a00deea65e9b1936924caaea8f44d476014856b68736")
                .unwrap(),
        )
        .unwrap();
        let node_address = NodeAddress {
            socket_addr,
            node_id,
        };

        let ip_target: Ipv4Addr = "127.0.0.1".parse().unwrap();
        let port_target = 5001;
        let socket_addr_target: SocketAddr = SocketAddrV4::new(ip_target, port_target).into();
        let node_id_target = NodeId::parse(
            &hex::decode("fb757dc581730490a1d7a00deea65e9b1936924caaea8f44d47601485668").unwrap(),
        )
        .unwrap();
        let node_address_target = NodeAddress {
            socket_addr: socket_addr_target,
            node_id: node_id_target,
        };

        let nonce_bytes = hex::decode("47644922f5d6e951051051ac").unwrap();
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        let notif = RelayInit(node_address, nonce, node_address_target);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }

    #[test]
    fn test_enocde_decode_relay_msg() {
        let ip: Ipv4Addr = "127.0.0.1".parse().unwrap();
        let port = 5000;
        let socket_addr: SocketAddr = SocketAddrV4::new(ip, port).into();
        let node_id = NodeId::parse(
            &hex::decode("fb757dc581730490a1d7a00deea65e9b193691111111111118").unwrap(),
        )
        .unwrap();
        let node_address = NodeAddress {
            socket_addr,
            node_id,
        };

        let nonce_bytes = hex::decode("9951051051aceb").unwrap();
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

        let notif = RelayMsg(node_address, nonce);

        let encoded_notif = notif.clone().rlp_encode();
        let decoded_notif = Notification::rlp_decode(&encoded_notif).expect("Should decode");

        assert_eq!(notif, decoded_notif.into());
    }
}
