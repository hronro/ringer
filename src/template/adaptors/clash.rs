use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use serde_yaml::to_string;

use crate::node::ss::{ObfsOpts, ObfsType, Plugin as SsPlugin};
use crate::node::{GetNodeName, Node};

use super::Adaptor;

/// Clash Proxy Configuration
/// Reference: https://github.com/Dreamacro/clash/wiki/Configuration
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClashProxy {
    #[serde(rename = "ss", rename_all = "kebab-case")]
    Ss {
        name: String,
        server: String,
        port: u16,
        cipher: String,
        password: String,
        udp: Option<bool>,
        #[serde(flatten)]
        plugin: Option<ClashSsPlugin>,
    },

    #[serde(rename = "ssr", rename_all = "kebab-case")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "plugin", rename_all = "kebab-case")]
pub enum ClashSsPlugin {
    Obfs { plugin_opts: ClashSsPluginObfsOpts }, // TODO: add V2Ray plugin
                                                 // V2rayPlugin {}
}
impl TryFrom<SsPlugin> for ClashSsPlugin {
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
impl From<ClashSsPlugin> for SsPlugin {
    fn from(value: ClashSsPlugin) -> Self {
        match value {
            ClashSsPlugin::Obfs { plugin_opts } => Self::SimpleObfs(plugin_opts.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClashSsPluginObfsOpts {
    pub mode: Option<ClashSsPluginObfsType>,
    pub host: Option<String>,
}
impl TryFrom<ObfsOpts> for ClashSsPluginObfsOpts {
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
impl From<ClashSsPluginObfsOpts> for ObfsOpts {
    fn from(value: ClashSsPluginObfsOpts) -> Self {
        Self {
            obfs: value.mode.map(Into::into),
            host: value.host,
            uri: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClashSsPluginObfsType {
    Http,
    Tls,
}
impl From<ObfsType> for ClashSsPluginObfsType {
    fn from(value: ObfsType) -> Self {
        match value {
            ObfsType::Http => Self::Http,
            ObfsType::Tls => Self::Tls,
        }
    }
}
impl From<ClashSsPluginObfsType> for ObfsType {
    fn from(value: ClashSsPluginObfsType) -> Self {
        match value {
            ClashSsPluginObfsType::Http => Self::Http,
            ClashSsPluginObfsType::Tls => Self::Tls,
        }
    }
}

#[derive(Default)]
pub struct Clash;
impl Adaptor for Clash {
    const ADAPTOR_NAME: &'static str = "clash";

    type Node<'a> = ClashProxy;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>> {
        match node {
            Node::Ss(ss_node) => {
                if ss_node.method.is_aead_2022_cipher() {
                    None
                } else if let Some(plugin) = &ss_node.plugin {
                    if let Ok(clash_ss_plugin) = plugin.clone().try_into() {
                        Some(ClashProxy::Ss {
                            name: ss_node.get_display_name(),
                            server: ss_node.server.clone(),
                            port: ss_node.server_port,
                            cipher: ss_node.method.get_alias().to_string(),
                            password: ss_node.password.clone(),
                            udp: ss_node.udp,
                            plugin: Some(clash_ss_plugin),
                        })
                    } else {
                        None
                    }
                } else {
                    Some(ClashProxy::Ss {
                        name: ss_node.get_display_name(),
                        server: ss_node.server.clone(),
                        port: ss_node.server_port,
                        cipher: ss_node.method.get_alias().to_string(),
                        password: ss_node.password.clone(),
                        udp: ss_node.udp,
                        plugin: None,
                    })
                }
            }

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

            Node::Hysteria(_) => None,

            Node::Wireguard(_) => None,
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
