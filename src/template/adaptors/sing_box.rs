use serde::Serialize;
use serde_json::to_string_pretty;
use serde_with::skip_serializing_none;

use crate::node::{GetNodeName, Node};

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
        network: Option<&'a str>,
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
        network: Option<&'a str>,
    },
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
                plugin: ss_node.plugin.as_deref(),
                plugin_opts: ss_node.plugin_opts_str(),
                // if `ss_node.udp` equals `Some(false)`,
                // the `network` should be `"tcp"`,
                // otherwise keep `network` as `None`.
                network: ss_node
                    .udp
                    .and_then(|udp| if udp { None } else { Some("tcp") }),
                udp_over_tcp: None,
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
        }
    }

    fn serialize_nodes<'a, T: Iterator<Item = Self::Node<'a>>>(
        &self,
        nodes: T,
        options: super::NodesSerializationOptions,
    ) -> String {
        let nodes: Vec<_> = nodes.collect();
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
