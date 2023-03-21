use enr::NodeId;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

/// Discv5 node address.
pub struct NodeAddress {
    pub socket_addr: SocketAddr,
    pub node_id: NodeId,
}

impl Encodable for NodeAddress {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2);
        match self.socket_addr.ip() {
            IpAddr::V4(addr) => s.append(&(&addr.octets() as &[u8])),
            IpAddr::V6(addr) => s.append(&(&addr.octets() as &[u8])),
        };
        s.append(&self.socket_addr.port());
    }
}

impl Decodable for NodeAddress {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        if !rlp.is_list() {
            return Err(DecoderError::RlpExpectedToBeList);
        }
        let list_len = rlp.item_count()?;
        if list_len < 2 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let socket_bytes = rlp.val_at::<Vec<u8>>(0)?;
        let mut port_bytes = [0u8; 2];
        port_bytes.copy_from_slice(&socket_bytes[socket_bytes.len() - 2..]);
        let port = u16::from_be_bytes(port_bytes);
        let socket_addr = match socket_bytes.len() {
            6 => {
                let mut ip = [0u8; 4];
                ip.copy_from_slice(&socket_bytes[..4]);
                let ipv4 = Ipv4Addr::from(ip);
                SocketAddr::V4(SocketAddrV4::new(ipv4, port))
            }
            18 => {
                let mut ip = [0u8; 16];
                ip.copy_from_slice(&socket_bytes[..16]);
                let ipv6 = Ipv6Addr::from(ip);
                SocketAddr::V6(SocketAddrV6::new(ipv6, port, 0, 0))
            }
            _ => {
                return Err(DecoderError::RlpIncorrectListLen);
            }
        };
        let node_id_bytes = rlp.val_at::<Vec<u8>>(1)?;
        if node_id_bytes.len() > 32 {
            return Err(DecoderError::RlpIsTooBig);
        }
        let mut node_id = [0u8; 32];
        node_id[32 - node_id_bytes.len()..].copy_from_slice(&node_id_bytes);
        let node_id = NodeId::from(node_id);

        Ok(Self {
            socket_addr,
            node_id,
        })
    }
}
