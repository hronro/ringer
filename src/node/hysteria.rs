use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::node::common::TlsOptions;

/// The configuration of a Hysteria node.
/// Reference: https://v1.hysteria.network/docs/advanced-usage/#client
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HysteriaNode {
    pub remarks: Option<String>,
    pub server: String,
    pub port: ServerPort,
    pub protocol: Option<Protocol>,
    pub up: Speed,
    pub down: Speed,
    pub obfs: Option<String>,
    pub auth: Option<String>,
    pub tls: TlsOptions,
}
impl super::GetNodeName for HysteriaNode {
    fn get_name(&self) -> Option<&String> {
        self.remarks.as_ref()
    }

    fn get_server(&'_ self) -> &'_ String {
        &self.server
    }

    fn get_port(&self) -> u16 {
        self.port.get_start_port()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServerPort {
    Single(u16),
    Range(u16, u16),
}
impl ServerPort {
    #[allow(dead_code)]
    pub fn is_single(&self) -> bool {
        match self {
            Self::Single(_) => true,
            Self::Range(_, _) => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_range(&self) -> bool {
        match self {
            Self::Single(_) => false,
            Self::Range(_, _) => true,
        }
    }

    pub fn get_start_port(&self) -> u16 {
        match self {
            Self::Single(port) => *port,
            Self::Range(start, _) => *start,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Protocol {
    #[serde(rename = "udp")]
    Udp,
    #[serde(rename = "wechat-video")]
    WechatVideo,
    #[serde(rename = "faketcp")]
    FakeTcp,
}
impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Udp => write!(f, "udp"),
            Self::WechatVideo => write!(f, "wechat-video"),
            Self::FakeTcp => write!(f, "faketcp"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Speed {
    Text(String),
    Mbps(u32),
}
impl Speed {
    #[allow(dead_code)]
    pub fn to_mbps(&self) -> Option<u32> {
        match self {
            Self::Text(text) => {
                for b in [" bps", "bps", " b", "b"] {
                    if text.ends_with(b) {
                        return text
                            .trim_end_matches(b)
                            .parse::<u64>()
                            .ok()
                            .map(|n| (n / 1024 / 1024) as u32);
                    }
                }

                for k in [" kbps", "kbps", " kb", "kb", " k", "k"] {
                    if text.ends_with(k) {
                        return text
                            .trim_end_matches(k)
                            .parse::<u64>()
                            .ok()
                            .map(|n| (n / 1024) as u32);
                    }
                }

                for m in [" mbps", "mbps", " mb", "mb", " m", "m"] {
                    if text.ends_with(m) {
                        return text.trim_end_matches(m).parse::<u32>().ok();
                    }
                }

                for g in [" gbps", "gbps", " gb", "gb", " g", "g"] {
                    if text.ends_with(g) {
                        return text
                            .trim_end_matches(g)
                            .parse::<u32>()
                            .ok()
                            .map(|n| n * 1024);
                    }
                }

                for t in [" tbps", "tbps", " tb", "tb", " t", "t"] {
                    if text.ends_with(t) {
                        return text
                            .trim_end_matches(t)
                            .parse::<u32>()
                            .ok()
                            .map(|n| n * 1024 * 1024);
                    }
                }

                None
            }

            Self::Mbps(mbps) => Some(*mbps),
        }
    }

    #[allow(dead_code)]
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Mbps(mbps) => format!("{mbps} mbps"),
        }
    }
}
