use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_yaml::to_string;

use crate::node::{GetNodeName, Node};

use super::Adaptor;

/// Clash Proxy Configuration
/// Reference: https://github.com/Dreamacro/clash/wiki/Configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClashProxy {
    #[serde(rename = "ss", rename_all = "snake_case")]
    Ss {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        udp: Option<bool>,
        plugin: Option<String>,
        plugin_opts: Option<BTreeMap<String, String>>,
    },
    #[serde(rename = "ssr", rename_all = "snake_case")]
    Ssr {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        obfs: String,
        obfs_param: Option<String>,
        protocol: String,
        protocol_param: Option<String>,
        udp: Option<bool>,
    },
}

#[derive(Default)]
pub struct Clash;
impl Adaptor for Clash {
    const ADAPTOR_NAME: &'static str = "clash";

    type Node<'a> = ClashProxy;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>> {
        match node {
            Node::Ss(ss_node) => Some(ClashProxy::Ss {
                name: ss_node.get_display_name(),
                server: ss_node.server.clone(),
                port: ss_node.server_port,
                cipher: ss_node.method.get_alias().to_string(),
                password: ss_node.password.clone(),
                udp: ss_node.udp,
                plugin: ss_node
                    .plugin
                    .as_ref()
                    .map(|plugin| plugin.plugin_name().to_string()),
                plugin_opts: ss_node.plugin.as_ref().map(|plugin| plugin.get_opts_map()),
            }),
            Node::Ssr(ssr_node) => Some(ClashProxy::Ssr {
                name: ssr_node.get_display_name(),
                server: ssr_node.server.clone(),
                port: ssr_node.server_port,
                cipher: ssr_node.method.clone(),
                password: ssr_node.password.clone(),
                obfs: ssr_node.obfs.clone(),
                obfs_param: ssr_node.obfs_param.clone(),
                protocol: ssr_node.protocol.clone(),
                protocol_param: ssr_node.protocol_param.clone(),
                udp: None,
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
