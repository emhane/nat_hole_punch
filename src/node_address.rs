use enr::NodeId;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

const DEFAULT_IPV6_FLOW_INFO: u32 = 0;
const DEFAULT_IPV6_SCOPE_ID: u32 = 0;

/// Discv5 node address.
#[derive(Clone, PartialEq, Debug)]
pub struct NodeAddress {
    /// An ipv4 or ipv6 socket. In the case of an ipv6 socket, the flow info and scope id are
    /// disregarded in the rlp encoding, as is done for crate [`enr`] and defaults
    /// [`DEFAULT_IPV6_FLOW_INFO`] and [`DEFAULT_IPV6_SCOPE_ID`] are always used.
    pub socket_addr: SocketAddr,
    pub node_id: NodeId,
}

impl Encodable for NodeAddress {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        match self.socket_addr.ip() {
            IpAddr::V4(addr) => s.append(
                &[
                    &addr.octets() as &[u8],
                    &self.socket_addr.port().to_be_bytes(),
                ]
                .concat(),
            ),
            IpAddr::V6(addr) => s.append(
                &[
                    &addr.octets() as &[u8],
                    &self.socket_addr.port().to_be_bytes(),
                ]
                .concat(),
            ),
        };
        s.append(&(&self.node_id.raw() as &[u8]));
    }
}

impl Decodable for NodeAddress {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        if !rlp.is_list() {
            return Err(DecoderError::RlpExpectedToBeList);
        }
        let list_len = rlp.item_count()?;
        if list_len < 2 {
            println!("list len {}", list_len);
            return Err(DecoderError::RlpIncorrectListLen);
        }
        let socket_bytes = rlp.val_at::<Vec<u8>>(0)?;
        let mut port_bytes = [0u8; 2];
        port_bytes.copy_from_slice(&socket_bytes[socket_bytes.len() - 2..]);
        let port = u16::from_be_bytes(port_bytes);
        println!("socket_bytes_len {}", socket_bytes.len());
        let socket_addr = match socket_bytes.len() {
            6 => {
                // 4 bytes ip + 2 bytes port
                let mut ip = [0u8; 4];
                ip.copy_from_slice(&socket_bytes[..4]);
                let ipv4 = Ipv4Addr::from(ip);
                SocketAddr::V4(SocketAddrV4::new(ipv4, port))
            }
            18 => {
                // 16 bytes ip + 2 bytes port
                let mut ip = [0u8; 16];
                ip.copy_from_slice(&socket_bytes[..16]);
                let ipv6 = Ipv6Addr::from(ip);
                SocketAddr::V6(SocketAddrV6::new(
                    ipv6,
                    port,
                    DEFAULT_IPV6_FLOW_INFO,
                    DEFAULT_IPV6_SCOPE_ID,
                ))
            }
            _ => {
                println!("list len {}", list_len);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enocde_decode() {
        let ip: Ipv6Addr = "::1".parse().unwrap();
        let port = 9030;
        let socket_addr: SocketAddr =
            SocketAddrV6::new(ip, port, DEFAULT_IPV6_FLOW_INFO, DEFAULT_IPV6_SCOPE_ID).into();
        let node_id = NodeId::parse(
            &hex::decode("7f9701f05473580f50caff3c8a62e97f380497a17405b9db2d30e3e91e7e4241")
                .unwrap(),
        )
        .unwrap();

        let node_address = NodeAddress {
            socket_addr,
            node_id,
        };

        let encoded_node_address = rlp::encode(&node_address).to_vec();
        let decoded_node_address = rlp::decode(&encoded_node_address).expect("Should decode");

        assert_eq!(node_address, decoded_node_address);
    }
}
