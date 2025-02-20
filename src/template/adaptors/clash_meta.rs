use base64_simd::STANDARD as base64;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use serde_yaml::to_string;

use crate::node::hysteria::{ServerPort as HysteriaServerPort, Speed as HysteriaSpeed};
use crate::node::hysteria2::{
    Obfuscation as Hysteria2Obfuscation, ServerPort as Hysteria2ServerPort,
};

use crate::node::ss::{ObfsOpts, ObfsType, Plugin as SsPlugin};
use crate::node::{GetNodeName, Node};

use super::Adaptor;

/// Clash.Meta Proxy Configuration
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
        cipher: &'a str,
        password: &'a str,
        udp: Option<bool>,
        #[serde(flatten)]
        plugin: Option<ClashMetaSsPlugin>,
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
        ports: Option<String>,
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

    #[serde(rename = "hysteria", rename_all = "kebab-case")]
    Hysteria2 {
        name: String,
        server: &'a str,
        port: u16,
        ports: Option<String>,
        password: Option<&'a str>,
        obfs: Option<&'a str>,
        obfs_password: Option<&'a str>,
        sni: Option<&'a str>,
        apln: Option<&'a [String]>,
        skip_cert_verify: Option<bool>,
        up: Option<String>,
        down: Option<String>,
    },

    #[serde(rename = "wireguard", rename_all = "kebab-case")]
    Wireguard {
        name: String,
        server: &'a str,
        port: u16,
        ip: Option<String>,
        ipv6: Option<String>,
        private_key: &'a str,
        public_key: &'a str,
        pre_shared_key: Option<&'a str>,
        reserved: Option<String>,
        mtu: Option<u32>,
        udp: Option<bool>,
        persistent_keepalive: Option<u32>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "plugin", rename_all = "kebab-case")]
pub enum ClashMetaSsPlugin {
    Obfs {
        plugin_opts: ClashMetaSsPluginObfsOpts,
    }, // TODO: add V2Ray plugin
       // V2rayPlugin {}
}
impl TryFrom<SsPlugin> for ClashMetaSsPlugin {
    type Error = ();

    fn try_from(value: SsPlugin) -> Result<Self, Self::Error> {
        match value {
            SsPlugin::SimpleObfs(obfs_opts) => {
                if let Ok(opts) = obfs_opts.try_into() {
                    Ok(Self::Obfs { plugin_opts: opts })
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}
impl From<ClashMetaSsPlugin> for SsPlugin {
    fn from(value: ClashMetaSsPlugin) -> Self {
        match value {
            ClashMetaSsPlugin::Obfs { plugin_opts } => Self::SimpleObfs(plugin_opts.into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClashMetaSsPluginObfsOpts {
    pub mode: Option<ClashMetaSsPluginObfsType>,
    pub host: Option<String>,
}
impl TryFrom<ObfsOpts> for ClashMetaSsPluginObfsOpts {
    type Error = ConvertObfsOptsToClashObfsOptsError;

    fn try_from(value: ObfsOpts) -> Result<Self, Self::Error> {
        if value.uri.is_some() {
            Err(ConvertObfsOptsToClashObfsOptsError::UriUnsupported)
        } else {
            Ok(Self {
                mode: value.obfs.map(Into::into),
                host: value.host,
            })
        }
    }
}
pub enum ConvertObfsOptsToClashObfsOptsError {
    /// Clash do not support `obfs-uri`
    UriUnsupported,
}
impl From<ClashMetaSsPluginObfsOpts> for ObfsOpts {
    fn from(value: ClashMetaSsPluginObfsOpts) -> Self {
        Self {
            obfs: value.mode.map(Into::into),
            host: value.host,
            uri: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClashMetaSsPluginObfsType {
    Http,
    Tls,
}
impl From<ObfsType> for ClashMetaSsPluginObfsType {
    fn from(value: ObfsType) -> Self {
        match value {
            ObfsType::Http => Self::Http,
            ObfsType::Tls => Self::Tls,
        }
    }
}
impl From<ClashMetaSsPluginObfsType> for ObfsType {
    fn from(value: ClashMetaSsPluginObfsType) -> Self {
        match value {
            ClashMetaSsPluginObfsType::Http => Self::Http,
            ClashMetaSsPluginObfsType::Tls => Self::Tls,
        }
    }
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
            Node::Ss(ss_node) => {
                if let Some(plugin) = &ss_node.plugin {
                    if let Ok(clash_ss_plugin) = plugin.clone().try_into() {
                        Some(ClashMetaProxy::Ss {
                            name: ss_node.get_display_name(),
                            server: &ss_node.server,
                            port: ss_node.server_port,
                            cipher: ss_node.method.get_alias(),
                            password: &ss_node.password,
                            udp: ss_node.udp,
                            plugin: Some(clash_ss_plugin),
                        })
                    } else {
                        None
                    }
                } else {
                    Some(ClashMetaProxy::Ss {
                        name: ss_node.get_display_name(),
                        server: &ss_node.server,
                        port: ss_node.server_port,
                        cipher: ss_node.method.get_alias(),
                        password: &ss_node.password,
                        udp: ss_node.udp,
                        plugin: None,
                    })
                }
            }

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
                port: hysteria_node.port.get_start_port(),
                ports: match &hysteria_node.port {
                    HysteriaServerPort::Single(_) => None,
                    HysteriaServerPort::Range(start, end) => Some(format!("{start}-{end}")),
                },
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

            Node::Hysteria2(hysteria2_node) => Some(ClashMetaProxy::Hysteria2 {
                name: hysteria2_node.get_display_name(),
                server: &hysteria2_node.server,
                port: hysteria2_node.port.get_start_port(),
                ports: match &hysteria2_node.port {
                    Hysteria2ServerPort::Single(_) => None,
                    Hysteria2ServerPort::Range(start, end) => Some(format!("{start}-{end}")),
                },
                password: hysteria2_node.auth.as_deref(),
                obfs: hysteria2_node.obfs.as_ref().map(|obfs| match obfs {
                    Hysteria2Obfuscation::Salamander { .. } => "salamander",
                }),
                obfs_password: hysteria2_node.obfs.as_ref().map(|obfs| match obfs {
                    Hysteria2Obfuscation::Salamander { password } => password.as_ref(),
                }),
                sni: hysteria2_node.tls.sni.as_deref(),
                apln: hysteria2_node.tls.alpn.as_deref(),
                skip_cert_verify: hysteria2_node.tls.insecure,
                up: hysteria2_node.up.as_ref().map(|up| up.to_text()),
                down: hysteria2_node.down.as_ref().map(|down| down.to_text()),
            }),

            Node::Wireguard(wireguard_node) => Some(ClashMetaProxy::Wireguard {
                name: wireguard_node.get_display_name(),
                server: &wireguard_node.server,
                port: wireguard_node.port,
                ip: wireguard_node.ip.map(|ip| ip.to_string()),
                ipv6: wireguard_node.ipv6.map(|ipv6| ipv6.to_string()),
                private_key: &wireguard_node.private_key,
                public_key: &wireguard_node.public_key,
                pre_shared_key: wireguard_node.pre_shared_key.as_deref(),
                reserved: wireguard_node
                    .reserved
                    .map(|reserved| base64.encode_to_string(reserved)),
                mtu: None,
                udp: None,
                persistent_keepalive: None,
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
