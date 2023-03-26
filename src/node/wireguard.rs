use std::net::{Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

/// The configuration of a Hysteria node.
/// Reference: https://www.wireguard.com/papers/wireguard.pdf
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct WireguardNode {
    /// The node name.
    pub remarks: Option<String>,

    /// The server address.
    pub server: String,

    /// The server port.
    pub port: u16,

    /// The IPv4 address used in the client.
    pub ip: Option<Ipv4Addr>,

    /// The IPv6 address used in the client.
    pub ipv6: Option<Ipv6Addr>,

    /// Private key of the client.
    pub private_key: String,

    /// Public key of the peer.
    pub public_key: String,

    /// Pre-shared key
    pub pre_shared_key: Option<String>,

    /// The reserved field in the Wireguard messages.
    /// In standard implementations of Wireguard,
    /// this should all be `[0, 0, 0]`.
    /// However, in some modified implementations (e.g. Cluodflare WARP),
    /// this field is required.
    pub reserved: Option<[u8; 3]>,
}
impl super::GetNodeName for WireguardNode {
    fn get_name(&self) -> Option<&String> {
        self.remarks.as_ref()
    }

    fn get_server(&'_ self) -> &'_ String {
        &self.server
    }

    fn get_port(&self) -> u16 {
        self.port
    }
}
