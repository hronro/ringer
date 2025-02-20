use std::fmt::Display;

use crate::node::ss::{ObfsOpts, Plugin as SsPlugin};
use crate::node::{GetNodeName, Node};
use crate::template::functions::gen_wireguard_node_id;

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
        obfs: Option<&'a ObfsOpts>,
        udp_relay: bool,
    },

    Hysteria2 {
        host: &'a str,
        port: u16,
        password: &'a str,
        download_bandwidth: Option<u32>,
    },

    Wireguard {
        section_name: String,
    },
}
impl Display for ProxyType<'_> {
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

                    if let Some(host) = &obfs.host {
                        write!(f, ", obfs-host={host}")?;
                    }

                    if let Some(uri) = &obfs.uri {
                        write!(f, ", obfs-uri={uri}")?;
                    }
                }
            }

            Self::Hysteria2 {
                host,
                port,
                password,
                download_bandwidth,
            } => {
                write!(f, "hysteria2, {host}, {port}, password={password}")?;

                if let Some(download_bandwidth) = download_bandwidth {
                    write!(f, ", download-bandwidth={download_bandwidth}")?;
                }
            }

            Self::Wireguard { section_name } => {
                write!(f, "wireguard, section-name={section_name}")?;
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
                    if let SsPlugin::SimpleObfs(obfs_opts) = plugin {
                        Some(obfs_opts)
                    } else {
                        return None;
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

            Node::Hysteria(_) => None,

            Node::Hysteria2(hysteria2_node) => Some(SurgeProxy {
                name: hysteria2_node.get_display_name(),
                proxy: ProxyType::Hysteria2 {
                    // Surge doesn't support manually specify IP address and SNI,
                    // only host is supported, so first try to use SNI as host,
                    // then fallback to `server` field.
                    host: hysteria2_node
                        .tls
                        .sni
                        .as_ref()
                        .unwrap_or(&hysteria2_node.server),
                    port: hysteria2_node.get_port(),
                    password: hysteria2_node.auth.as_ref().map_or("", |password| password),
                    download_bandwidth: hysteria2_node
                        .down
                        .as_ref()
                        .and_then(|down| down.to_mbps()),
                },
            }),

            Node::Wireguard(wireguard_node) => Some(SurgeProxy {
                name: wireguard_node.get_display_name(),
                proxy: ProxyType::Wireguard {
                    section_name: gen_wireguard_node_id(wireguard_node),
                },
            }),
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
