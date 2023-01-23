use std::fmt::Display;

use crate::node::ss::{parse_obfs_plugin_args, ObfsArgs};
use crate::node::{GetNodeName, Node};

use super::Adaptor;

/// Surge Proxy Policy
/// Reference: https://manual.nssurge.com/policy/proxy.html
pub struct SurgeProxy<'a> {
    name: String,
    proxy: ProxyType<'a>,
}

pub enum ProxyType<'a> {
    Ss {
        host: &'a str,
        port: u16,
        encrypt_method: &'a str,
        password: &'a str,
        obfs: Option<ObfsArgs<'a>>,
        udp_relay: bool,
    },
}
impl<'a> Display for ProxyType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ss {
                host,
                port,
                encrypt_method,
                password,
                obfs,
                udp_relay,
            } => {
                write!(
                    f,
                    "ss, {host}, {port}, encrypt-method={encrypt_method}, password={password}, udp-relay={udp_relay}"
                )?;

                if let Some(obfs) = obfs {
                    if let Some(obfs_obfs) = obfs.obfs {
                        write!(f, ", obfs={obfs_obfs}")?;
                    }

                    if let Some(host) = obfs.host {
                        write!(f, ", obfs-host={host}")?;
                    }

                    if let Some(uri) = obfs.uri {
                        write!(f, ", obfs-uri={uri}")?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Surge;
impl Adaptor for Surge {
    const ADAPTOR_NAME: &'static str = "surge";

    type Node<'a> = SurgeProxy<'a>;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>> {
        match node {
            Node::Ss(ss_node) => {
                let obfs = if let Some(plugin) = &ss_node.plugin {
                    if plugin != "simple-obfs" {
                        return None;
                    }

                    match ss_node.plugin_opts.as_ref().map(parse_obfs_plugin_args) {
                        Some(Ok(obfs_args)) => Some(obfs_args),
                        Some(Err(_)) => {
                            return None;
                        }
                        None => None,
                    }
                } else {
                    None
                };

                Some(SurgeProxy {
                    name: ss_node.get_display_name(),
                    proxy: ProxyType::Ss {
                        host: &ss_node.server,
                        port: ss_node.server_port,
                        encrypt_method: ss_node.method.get_alias(),
                        password: &ss_node.password,
                        obfs,
                        // UDP relay should be `false` when `udp_over_tcp` is `true`,
                        // since Surge doesn't support `udp_over_tcp`.
                        udp_relay: matches!(&ss_node.udp, Some(true) if !matches!(ss_node.udp_over_tcp, Some(true))),
                    },
                })
            }

            Node::Ssr(_) => None,
        }
    }

    fn serialize_nodes<'a, T: Iterator<Item = Self::Node<'a>>>(
        &self,
        nodes: T,
        _options: super::NodesSerializationOptions,
    ) -> String {
        nodes
            .into_iter()
            .map(|node| format!("{} = {}", node.name, node.proxy))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
