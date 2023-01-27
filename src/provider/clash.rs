use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use http::Uri;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::node::ss::{Method as SsMethod, Plugin as SsPlugin};
use crate::node::{Node, SsNode, SsrNode};
use crate::template::adaptors::clash::ClashProxy;

use super::{CommonProviderOptions, Provider};

/// SSR subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clash {
    /// Name of the Clash subscription.
    pub name: Option<String>,

    /// URL of the Clash subscription.
    #[serde(with = "http_serde::uri")]
    pub url: Uri,

    /// Common provider options.
    #[serde(flatten)]
    pub options: CommonProviderOptions,
}

#[async_trait]
impl Provider for Clash {
    fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn get_url(&self) -> &Uri {
        &self.url
    }

    fn set_url(&mut self, url: Uri) {
        self.url = url;
    }

    fn parse_nodes_from_content(&self, content: Bytes) -> Result<Vec<Node>> {
        let clash_config: ClashConfiguration = serde_yaml::from_slice(&content)?;
        Ok(clash_config
            .proxies
            .into_iter()
            .filter_map(|ipnoupn| match ipnoupn {
                ImplementedProxyNodeOrUnknownProxyNode::Implemented(clash_proxy) => {
                    Some(clash_proxy)
                }
                ImplementedProxyNodeOrUnknownProxyNode::Unknown(_) => None,
            })
            .filter_map(|proxy| match proxy {
                ClashProxy::Ss {
                    name,
                    server,
                    port,
                    cipher,
                    password,
                    udp,
                    plugin: plugin_name,
                    plugin_opts,
                } => {
                    let method = SsMethod::from_alias(&cipher)?;

                    let plugin = match plugin_name.map(|plugin_name| {
                        SsPlugin::from_name_and_opts(plugin_name, plugin_opts.unwrap_or_default())
                    }) {
                        Some(Ok(plugin)) => Some(plugin),
                        Some(Err(_)) => {
                            warn!("Invalid SS plugin in `{}`, skip it.", &name);
                            return None;
                        }
                        None => None,
                    };
                    Some(Node::Ss(Box::new(SsNode {
                        id: None,
                        remarks: Some(name),
                        server,
                        server_port: port,
                        password,
                        method,
                        udp,
                        udp_over_tcp: None,
                        plugin,
                    })))
                }
                ClashProxy::Ssr {
                    name,
                    server,
                    port,
                    cipher,
                    password,
                    obfs,
                    obfs_param,
                    protocol,
                    protocol_param,
                    udp: _,
                } => Some(Node::Ssr(Box::new(SsrNode {
                    remarks: Some(name),
                    server,
                    server_port: port,
                    password,
                    method: cipher,
                    protocol,
                    protocol_param,
                    obfs,
                    obfs_param,
                    udpport: None,
                    uot: None,
                }))),
            })
            .collect())
    }
}

// TODO: Remove this after implemented all types of nodes.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ImplementedProxyNodeOrUnknownProxyNode {
    Implemented(ClashProxy),
    Unknown(Value),
}

#[derive(Debug, Deserialize)]
struct ClashConfiguration {
    proxies: Vec<ImplementedProxyNodeOrUnknownProxyNode>,
}
