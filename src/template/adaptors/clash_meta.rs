use std::collections::BTreeMap;

use serde::Serialize;
use serde_with::skip_serializing_none;
use serde_yaml::to_string;

use crate::node::{hysteria::Speed as HysteriaSpeed, GetNodeName, Node};

use super::Adaptor;

/// Clash Proxy Configuration
/// Reference: https://github.com/MetaCubeX/Clash.Meta/blob/Meta/docs/config.yaml
#[skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ClashMetaProxy<'a> {
    #[serde(rename = "ss", rename_all = "kebab-case")]
    Ss {
        name: String,
        server: &'a str,
        port: u16,
        cipher: String,
        password: &'a str,
        udp: Option<bool>,
        udp_over_tcp: Option<bool>,
        ip_version: Option<ClashMetaIpVersion>,
        plugin: Option<&'a str>,
        plugin_opts: Option<BTreeMap<String, String>>,
    },

    #[serde(rename = "ssr", rename_all = "kebab-case")]
    Ssr {
        name: String,
        server: &'a str,
        port: u16,
        cipher: &'a str,
        password: &'a str,
        obfs: &'a str,
        obfs_param: Option<&'a str>,
        protocol: &'a str,
        protocol_param: Option<&'a str>,
        udp: Option<bool>,
    },

    #[serde(rename = "hysteria", rename_all = "kebab-case")]
    Hysteria {
        name: String,
        server: &'a str,
        port: u16,
        auth_str: Option<&'a str>,
        obfs: Option<&'a str>,
        apln: Option<&'a [String]>,
        protocol: Option<String>,
        up: String,
        down: String,
        sni: Option<&'a str>,
        skip_cert_verify: Option<bool>,
        #[serde(rename = "disable_mtu_discovery")]
        disable_mtu_discovery: Option<bool>,
        fast_open: Option<bool>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClashMetaIpVersion {
    Dual,
    Ipv4,
    Ipv6,
    Ipv4Prefer,
    Ipv6Prefer,
}

#[derive(Default)]
pub struct ClashMeta;
impl Adaptor for ClashMeta {
    const ADAPTOR_NAME: &'static str = "clash meta";

    type Node<'a> = ClashMetaProxy<'a>;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>> {
        match node {
            Node::Ss(ss_node) => Some(ClashMetaProxy::Ss {
                name: ss_node.get_display_name(),
                server: &ss_node.server,
                port: ss_node.server_port,
                cipher: ss_node.method.get_alias().to_string(),
                password: &ss_node.password,
                udp: ss_node.udp,
                udp_over_tcp: ss_node.udp_over_tcp,
                ip_version: None,
                plugin: ss_node.plugin.as_ref().map(|plugin| plugin.plugin_name()),
                plugin_opts: ss_node.plugin.as_ref().map(|plugin| plugin.get_opts_map()),
            }),

            Node::Ssr(ssr_node) => Some(ClashMetaProxy::Ssr {
                name: ssr_node.get_display_name(),
                server: &ssr_node.server,
                port: ssr_node.server_port,
                cipher: &ssr_node.method,
                password: &ssr_node.password,
                obfs: &ssr_node.obfs,
                obfs_param: ssr_node.obfs_param.as_deref(),
                protocol: &ssr_node.protocol,
                protocol_param: ssr_node.protocol_param.as_deref(),
                udp: None,
            }),

            Node::Hysteria(hysteria_node) => Some(ClashMetaProxy::Hysteria {
                name: hysteria_node.get_display_name(),
                server: &hysteria_node.server,
                port: hysteria_node.port,
                auth_str: hysteria_node.auth.as_deref(),
                obfs: hysteria_node.obfs.as_deref(),
                apln: hysteria_node.tls.alpn.as_deref(),
                protocol: hysteria_node.protocol.map(|protocol| protocol.to_string()),
                up: match &hysteria_node.up {
                    HysteriaSpeed::Text(up) => up.clone(),
                    HysteriaSpeed::Mbps(up) => format!("{up} Mbps"),
                },
                down: match &hysteria_node.down {
                    HysteriaSpeed::Text(down) => down.clone(),
                    HysteriaSpeed::Mbps(down) => format!("{down} Mbps"),
                },
                sni: hysteria_node.tls.sni.as_deref(),
                skip_cert_verify: hysteria_node.tls.insecure,
                disable_mtu_discovery: None,
                fast_open: None,
            }),
        }
    }

    fn serialize_nodes<'a, T: Iterator<Item = Self::Node<'a>>>(
        &self,
        nodes: T,
        _options: super::NodesSerializationOptions,
    ) -> String {
        let nodes: Vec<_> = nodes.collect();
        to_string(&nodes).unwrap()
    }
}
