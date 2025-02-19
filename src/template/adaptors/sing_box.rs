use serde::Serialize;
use serde_json::to_string_pretty;
use serde_with::skip_serializing_none;

use crate::node::{hysteria::Speed as HysteriaSpeed, hysteria::ServerPort as HysteriaServerPort, hysteria2::ServerPort as Hysteria2ServerPort, hysteria2::Obfuscation as Hysteria2Obfuscation, ss::Plugin as SsPlugin, GetNodeName, Node};

use super::Adaptor;

/// sing-box outbound
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SingBoxNode<'a> {
    /// Shadowsocks outbound
    /// Reference: https://sing-box.sagernet.org/configuration/outbound/shadowsocks
    #[serde(rename = "shadowsocks")]
    Shadowsocks {
        tag: String,
        server: &'a str,
        server_port: u16,
        method: &'a str,
        password: &'a str,
        plugin: Option<&'a str>,
        plugin_opts: Option<String>,
        network: Option<&'static str>,
        udp_over_tcp: Option<bool>,
    },

    /// ShadowsocksR outbound
    /// Reference: https://sing-box.sagernet.org/configuration/outbound/shadowsocksr
    #[serde(rename = "shadowsocksr")]
    Shadowsocksr {
        tag: String,
        server: &'a str,
        server_port: u16,
        method: &'a str,
        password: &'a str,
        obfs: &'a str,
        obfs_param: Option<&'a str>,
        protocol: &'a str,
        protocol_param: Option<&'a str>,
        network: Option<&'static str>,
    },

    /// Hysteria outbound
    /// Reference: https://sing-box.sagernet.org/configuration/outbound/hysteria
    #[serde(rename = "hysteria")]
    Hysteria {
        tag: String,
        server: &'a str,
        server_port: Option<u16>,
        server_ports: Option<Vec<String>>,
        up: Option<&'a str>,
        up_mbps: Option<u32>,
        down: Option<&'a str>,
        down_mbps: Option<u32>,
        obfs: Option<&'a str>,
        auth_str: Option<&'a str>,
        tls: SingBoxTlsOptions<'a>,
    },

    /// Hysteria outbound
    /// Reference: https://sing-box.sagernet.org/configuration/outbound/hysteria2
    #[serde(rename = "hysteria2")]
    Hysteria2 {
        tag: String,
        server: &'a str,
        server_port: Option<u16>,
        server_ports: Option<Vec<String>>,
        up_mbps: Option<u32>,
        down_mbps: Option<u32>,
        obfs: Option<SingBoxHysteria2Obfuscation>,
        password: Option<&'a str>,
        tls: SingBoxTlsOptions<'a>,
    },

    /// Wireguard outbound
    /// Reference: https://sing-box.sagernet.org/configuration/outbound/wireguard
    #[serde(rename = "wireguard")]
    Wireguard {
        tag: String,
        server: &'a str,
        server_port: u16,
        system_interface: Option<bool>,
        interface_name: Option<String>,
        local_address: Vec<String>,
        private_key: &'a str,
        peer_public_key: &'a str,
        pre_shared_key: Option<&'a str>,
        reserved: Option<[u8; 3]>,
        workers: Option<u8>,
        mtu: Option<u32>,
        network: Option<&'static str>,
    },
}

/// TLS Options
/// Reference: https://sing-box.sagernet.org/configuration/shared/tls/#outbound
#[skip_serializing_none]
#[derive(Serialize)]
pub struct SingBoxTlsOptions<'a> {
    enabled: bool,
    server_name: Option<&'a str>,
    insecure: Option<bool>,
    alpn: Option<&'a [String]>,
}

/// Singbox Hysteria2 Obfuscation
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SingBoxHysteria2Obfuscation {
    #[serde(rename = "salamander")]
    Salamander {
        password: String,
    }
}

#[derive(Default)]
pub struct SingBox;
impl Adaptor for SingBox {
    const ADAPTOR_NAME: &'static str = "sing-box";

    type Node<'a> = SingBoxNode<'a>;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>> {
        match node {
            Node::Ss(ss_node) => Some(SingBoxNode::Shadowsocks {
                tag: ss_node.get_display_name(),
                server: &ss_node.server,
                server_port: ss_node.server_port,
                method: ss_node.method.get_alias(),
                password: &ss_node.password,
                // plugin: ss_node.plugin.as_deref(),
                plugin: match ss_node.plugin {
                    Some(SsPlugin::SimpleObfs(_)) => Some("obfs-local"),
                    Some(SsPlugin::V2ray) => Some("v2ray-plugin"),
                    None => None,

                    // Other plugins are not supported in sing-box.
                    _ => {
                        return None;
                    }
                },
                plugin_opts: ss_node
                    .plugin
                    .as_ref()
                    .and_then(|plugin| plugin.get_opts_string()),
                // if `ss_node.udp` equals `Some(false)`,
                // the `network` should be `"tcp"`,
                // otherwise keep `network` as `None`.
                network: ss_node
                    .udp
                    .and_then(|udp| if udp { None } else { Some("tcp") }),
                udp_over_tcp: ss_node.udp_over_tcp,
            }),

            Node::Ssr(ssr_node) => Some(SingBoxNode::Shadowsocksr {
                tag: ssr_node.get_display_name(),
                server: &ssr_node.server,
                server_port: ssr_node.server_port,
                method: &ssr_node.method,
                password: &ssr_node.password,
                obfs: &ssr_node.obfs,
                obfs_param: ssr_node.obfs_param.as_deref(),
                protocol: &ssr_node.protocol,
                protocol_param: ssr_node.protocol_param.as_deref(),
                network: None,
            }),

            Node::Hysteria(hysteria_node) => Some(SingBoxNode::Hysteria {
                tag: hysteria_node.get_display_name(),
                server: &hysteria_node.server,
                server_port: match hysteria_node.port {
                    HysteriaServerPort::Single(port) => Some(port),
                    _ => None,
                }, 
                server_ports: match hysteria_node.port {
                    HysteriaServerPort::Single(_) => None,
                    HysteriaServerPort::Range(start, end) => Some(
                        vec![format!("{start}:{end}")],
                    ),
                },
                up: match &hysteria_node.up {
                    HysteriaSpeed::Text(up) => Some(up),
                    HysteriaSpeed::Mbps(_) => None,
                },
                up_mbps: match &hysteria_node.up {
                    HysteriaSpeed::Text(_) => None,
                    HysteriaSpeed::Mbps(up) => Some(*up),
                },
                down: match &hysteria_node.down {
                    HysteriaSpeed::Text(down) => Some(down),
                    HysteriaSpeed::Mbps(_) => None,
                },
                down_mbps: match &hysteria_node.down {
                    HysteriaSpeed::Text(_) => None,
                    HysteriaSpeed::Mbps(down) => Some(*down),
                },
                obfs: hysteria_node.obfs.as_deref(),
                auth_str: hysteria_node.auth.as_deref(),
                tls: SingBoxTlsOptions {
                    enabled: true,
                    server_name: Some(
                        hysteria_node
                            .tls
                            .sni
                            .as_deref()
                            .unwrap_or(&hysteria_node.server),
                    ),
                    insecure: hysteria_node.tls.insecure,
                    alpn: hysteria_node.tls.alpn.as_deref(),
                },
            }),

            Node::Hysteria2(hysteria2_node) => Some(SingBoxNode::Hysteria2 {
                tag: hysteria2_node.get_display_name(),
                server: &hysteria2_node.server,
                server_port: match hysteria2_node.port {
                    Hysteria2ServerPort::Single(port) => Some(port),
                    _ => None,
                }, 
                server_ports: match hysteria2_node.port {
                    Hysteria2ServerPort::Single(_) => None,
                    Hysteria2ServerPort::Range(start, end) => Some(
                        vec![format!("{start}:{end}")],
                    ),
                },
                up_mbps: hysteria2_node.up.to_mbps(),
                down_mbps: hysteria2_node.down.to_mbps(),
                obfs: hysteria2_node.obfs.as_ref().map(|obfs| match obfs {
                    Hysteria2Obfuscation::Salamander { password } => SingBoxHysteria2Obfuscation::Salamander { password: password.clone() },
                }),
                password: hysteria2_node.auth.as_deref(),
                tls: SingBoxTlsOptions {
                    enabled: true,
                    server_name: Some(
                        hysteria2_node
                            .tls
                            .sni
                            .as_deref()
                            .unwrap_or(&hysteria2_node.server),
                    ),
                    insecure: hysteria2_node.tls.insecure,
                    alpn: hysteria2_node.tls.alpn.as_deref(),
                },
            }),

            Node::Wireguard(wireguard_node) => Some(SingBoxNode::Wireguard {
                tag: wireguard_node.get_display_name(),
                server: &wireguard_node.server,
                server_port: wireguard_node.port,
                system_interface: None,
                interface_name: None,
                local_address: [
                    wireguard_node.ip.map(|ip| format!("{ip}/32")),
                    wireguard_node.ipv6.map(|ipv6| format!("{ipv6}/128")),
                ]
                .into_iter()
                .flatten()
                .collect(),
                private_key: &wireguard_node.private_key,
                peer_public_key: &wireguard_node.public_key,
                pre_shared_key: wireguard_node.pre_shared_key.as_deref(),
                reserved: wireguard_node.reserved,
                workers: None,
                mtu: None,
                network: None,
            }),
        }
    }

    fn serialize_nodes<'a, T: Iterator<Item = Self::Node<'a>>>(
        &self,
        nodes: T,
        options: super::NodesSerializationOptions,
    ) -> String {
        let nodes: Vec<_> = nodes.collect();

        if nodes.is_empty() {
            return String::from("");
        }

        let mut output = to_string_pretty(&nodes).unwrap();

        if options.include_array_brackets {
            output
        } else {
            output.pop();
            output.pop();
            output.split_off(2)
        }
    }
}
