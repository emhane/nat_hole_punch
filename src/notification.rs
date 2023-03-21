use crate::{impl_from_variant_wrap, HolePunchError, NodeAddress};
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

impl Notification {
    pub fn rlp_decode(data: &[u8]) -> Result<Self, HolePunchError> {
        if data.len() < 3 {
            return Err(DecoderError::RlpIsTooShort.into());
        }
        let msg_type = data[0];

        let rlp = Rlp::new(&data[1..]);
        let list_len = rlp.item_count()?;
        if list_len < 2 {
            return Err(DecoderError::RlpIsTooShort.into());
        }
        let initiator = rlp.val_at::<NodeAddress>(0)?;
        let nonce_bytes = rlp.val_at::<Vec<u8>>(1)?;
        if nonce_bytes.len() > MESSAGE_NONCE_LENGTH {
            return Err(DecoderError::RlpIsTooBig.into());
        }
        let mut nonce = [0u8; MESSAGE_NONCE_LENGTH];
        nonce[MESSAGE_NONCE_LENGTH - nonce_bytes.len()..].copy_from_slice(&nonce_bytes);

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

    fn rlp_encode(self) -> Vec<u8> {
        match self {
            Notification::RelayInit(notif) => notif.rlp_encode(),
            Notification::RelayMsg(notif) => notif.rlp_encode(),
        }
    }
}

impl RelayInit {
    pub fn new(initiator: NodeAddress, nonce: MessageNonce, target: NodeAddress) -> Self {
        Self(initiator, nonce, target)
    }

    fn rlp_encode(self) -> Vec<u8> {
        let RelayInit(initiator, nonce, target) = self;

        let mut s = RlpStream::new();
        s.begin_list(3);
        s.append(&initiator);
        s.append(&nonce.to_vec());
        s.append(&target);

        let mut buf = Vec::with_capacity(4);
        buf.push(REALY_INIT_NOTIF_TYPE);
        buf.extend_from_slice(&s.out());
        buf
    }
}

impl RelayMsg {
    pub fn new(initiator: NodeAddress, nonce: MessageNonce) -> Self {
        Self(initiator, nonce)
    }

    fn rlp_encode(self) -> Vec<u8> {
        let RelayMsg(initiator, nonce) = self;

        let mut s = RlpStream::new();
        s.begin_list(3);
        s.append(&initiator);
        s.append(&nonce.to_vec());

        let mut buf = Vec::with_capacity(3);
        buf.push(REALY_MSG_NOTIF_TYPE);
        buf.extend_from_slice(&s.out());
        buf
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
        let port_target = 5000;
        let socket_addr_target: SocketAddr = SocketAddrV4::new(ip_target, port_target).into();
        let node_id_target = NodeId::parse(
            &hex::decode("fb757dc581730490a1d7a00deea65e9b1936924caaea8f44d476014856b68736")
                .unwrap(),
        )
        .unwrap();
        let node_address_target = NodeAddress {
            socket_addr: socket_addr_target,
            node_id: node_id_target,
        };

        let message_nonce = todo!();

        //let notif = Notification::RelayInit(RelayInit());
    }

    #[test]
    fn test_enocde_decode_relay_msg() {}
}
