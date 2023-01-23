use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use enum_dispatch::enum_dispatch;
use http::Uri;
use serde::{Deserialize, Serialize};

use crate::node::Node;
use crate::utils::{load_content_from_url, Path};

mod clash;
mod ssr;

pub use clash::Clash;
pub use ssr::Ssr;

#[async_trait]
#[enum_dispatch]
pub trait Provider {
    fn get_name(&self) -> Option<&str>;

    fn get_url(&self) -> &Uri;

    fn set_url(&mut self, url: Uri);

    fn parse_nodes_from_content(&self, content: Bytes) -> Result<Vec<Node>>;

    fn get_display_name(&self) -> String {
        self.get_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| self.get_url().to_string())
    }

    async fn fetch_content(&self) -> Result<Bytes> {
        load_content_from_url(Path::Url(self.get_url().clone())).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
#[enum_dispatch(Provider)]
pub enum Providers {
    Ssr(Ssr),
    Clash(Clash),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonProviderOptions {
    /// Override the `udp` field in all Shadowsocks nodes.
    pub ss_udp: Option<bool>,

    /// Override the `udp_over_tcp` field in all Shadowsocks nodes.
    pub ss_udp_over_tcp: Option<bool>,

    /// Override the `udpport` field in all ShadowsocksR nodes.
    pub ssr_udpport: Option<u16>,

    /// Override the `uot` field in all ShadowsocksR nodes.
    pub ssr_uot: Option<bool>,
}
