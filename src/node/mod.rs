use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

pub mod common;
pub mod hysteria;
pub mod hysteria2;
pub mod ss;
pub mod ssr;
pub mod wireguard;

pub use hysteria::HysteriaNode;
pub use hysteria2::Hysteria2Node;
pub use ss::SsNode;
pub use ssr::SsrNode;
pub use wireguard::WireguardNode;

#[enum_dispatch]
pub trait GetNodeName {
    fn get_name(&self) -> Option<&String>;

    fn get_server(&'_ self) -> &'_ String;

    fn get_port(&self) -> u16;

    fn get_display_name(&self) -> String {
        self.get_name()
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("{}:{}", self.get_server(), self.get_port()))
    }
}
impl<T> GetNodeName for Box<T>
where
    T: GetNodeName,
{
    fn get_name(&self) -> Option<&String> {
        self.as_ref().get_name()
    }

    fn get_server(&'_ self) -> &'_ String {
        self.as_ref().get_server()
    }

    fn get_port(&self) -> u16 {
        self.as_ref().get_port()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
#[enum_dispatch(GetNodeName)]
pub enum Node {
    #[serde(rename = "shadowsocks")]
    Ss(Box<SsNode>),
    #[serde(rename = "shadowsocksr")]
    Ssr(Box<SsrNode>),
    #[serde(rename = "hysteria")]
    Hysteria(Box<HysteriaNode>),
    #[serde(rename = "hysteria2")]
    Hysteria2(Box<Hysteria2Node>),
    #[serde(rename = "wireguard")]
    Wireguard(WireguardNode),
}
