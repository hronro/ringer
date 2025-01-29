use std::collections::HashMap;
use std::fmt::Write;
use std::hash::Hash;

use serde_json::Value;
use tera::Function;

use crate::node::wireguard::WireguardNode;
use crate::node::Node;
use crate::template::TemplateArgs;
use crate::utils::Blake3Hasher;

use super::{get_filtered_nodes_by_function_args, RingerFunctions};

pub fn gen_wireguard_node_id(node: &WireguardNode) -> String {
    let mut hasher = Blake3Hasher::new();
    node.hash(&mut hasher);
    hasher.get_hash().to_hex().to_string()
}

pub struct GetSurgeWgNodes<'a>(&'a TemplateArgs<'a>);
impl<'a> GetSurgeWgNodes<'a> {
    pub fn new(args: &'a TemplateArgs) -> Self {
        Self(args)
    }
}
impl RingerFunctions for GetSurgeWgNodes<'_> {
    const NAME: &'static str = "get_surge_wg_nodes";
}
impl Function for GetSurgeWgNodes<'_> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let nodes = get_filtered_nodes_by_function_args(Self::NAME, self.0, args)?;

        let surge_wg_nodes = nodes
            .filter_map(|node| {
                if let Node::Wireguard(wg_node) = node {
                    Some(wg_node)
                } else {
                    None
                }
            })
            .map(|wg_node| {
                let mut wg_node_string = format!(
                    "[WireGuard {}]\nprivate-key = {}",
                    gen_wireguard_node_id(wg_node),
                    wg_node.private_key,
                );

                if let Some(ip) = wg_node.ip {
                    write!(&mut wg_node_string, "\nself-ip = {}", ip).unwrap();
                }

                if let Some(ipv6) = wg_node.ipv6 {
                    write!(&mut wg_node_string, "\nself-ip-v6 = {}", ipv6).unwrap();
                }

                if let Some(reserved) = wg_node.reserved {
                    write!(
                        &mut wg_node_string,
                        "\npeer = (public-key = {}, allowed-ips = \"0.0.0.0/0, ::/0\", endpoint = {}:{}, client-id = {}/{}/{})",
                        wg_node.public_key,
                        wg_node.server,
                        wg_node.port,
                        reserved[0],
                        reserved[1],
                        reserved[2]
                    ).unwrap();
                } else {
                    write!(
                        &mut wg_node_string,
                        "\npeer = (public-key = {}, allowed-ips = \"0.0.0.0/0, ::/0\", endpoint = {}:{}",
                        wg_node.public_key,
                        wg_node.server,
                        wg_node.port,
                    ).unwrap();
                }

                wg_node_string
            })
            .collect::<Vec<String>>()
            .join("\n");

        Ok(Value::String(surge_wg_nodes))
    }

    fn is_safe(&self) -> bool {
        true
    }
}
