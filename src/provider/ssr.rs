use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64_simd::STANDARD as base64;
use base64_simd::STANDARD_NO_PAD as base64_no_pad;
use bytes::Bytes;
use http::Uri;
use log::{debug, trace};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::node::{Node, SsNode, SsrNode};

use super::{CommonProviderOptions, Provider};

/// SSR subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ssr {
    /// Name of the SSR subscription.
    pub name: Option<String>,

    /// URL of the SSR subscription.
    #[serde(with = "http_serde::uri")]
    pub url: Uri,

    /// Common provider options.
    #[serde(flatten)]
    pub options: CommonProviderOptions,
}

#[async_trait]
impl Provider for Ssr {
    fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    fn get_url(&self) -> &Uri {
        &self.url
    }

    fn set_url(&mut self, url: Uri) {
        self.url = url;
    }

    // Reference: https://github.com/shadowsocksr-backup/shadowsocks-rss/wiki/Subscribe-服务器订阅接口文档
    fn parse_nodes_from_content(&self, content: Bytes) -> Result<Vec<Node>> {
        let decoded_content = match base64
            .decode_to_vec(&content)
            .context("failed to decode base64 from the provider content")
        {
            Ok(decoded_content) => Ok(decoded_content),
            Err(e) => {
                // According to the SSR wiki, the base64 string should not be padded.
                // However, some providers do not follow this rule, so we need to try
                // to decode without padding.
                debug!("failed to decode SSR subscription, now try to decode without padding");
                if let Ok(decoded_content) = base64_no_pad.decode_to_vec(&content) {
                    Ok(decoded_content)
                } else {
                    Err(e)
                }
            }
        }?;
        let decoded_string = String::from_utf8_lossy(&decoded_content);
        trace!(
            "decoded content of provider `{}`:\n{:?}",
            self.url,
            &decoded_string
        );

        decoded_string
            .par_split('\n')
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }

                line.parse::<Url>().ok()
            })
            .map(|link| match link.scheme() {
                "ss" => {
                    let mut ss_node = SsNode::from_url(&link)
                        .with_context(|| format!("failed to parse SS node from url:\n{link}"))?;

                    if let Some(ss_udp) = self.options.ss_udp {
                        ss_node.udp = Some(ss_udp);
                    }

                    if let Some(ss_uot) = self.options.ss_udp_over_tcp {
                        ss_node.udp_over_tcp = Some(ss_uot);
                    }

                    Ok(Node::Ss(Box::new(ss_node)))
                }

                "ssr" => {
                    let mut ssr_node = SsrNode::from_url(&link)
                        .with_context(|| format!("failed to parse SSR node from URL:\n{link}"))?;

                    if let Some(ssr_udpport) = self.options.ssr_udpport {
                        ssr_node.udpport = Some(ssr_udpport);
                    }

                    if let Some(ssr_uot) = self.options.ssr_uot {
                        ssr_node.uot = Some(ssr_uot);
                    }

                    Ok(Node::Ssr(Box::new(ssr_node)))
                }

                _ => Err(anyhow!("Unknown scheme for `{}`", &link)),
            })
            .collect()
    }
}
