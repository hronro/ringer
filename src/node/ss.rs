use std::collections::BTreeMap;
use std::fmt::Display;

use anyhow::{anyhow, Result};
use base64_simd::URL_SAFE_NO_PAD as base64_url_no_pad;
use itertools::Itertools;
use log::trace;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// The configuration of a Shadowsocks node.
/// Reference: https://shadowsocks.org/guide/sip008.html
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SsNode {
    pub id: Option<Uuid>,
    pub remarks: Option<String>,
    pub server: String,
    pub server_port: u16,
    pub password: String,
    pub method: Method,
    pub udp: Option<bool>,
    pub udp_over_tcp: Option<bool>,
    pub plugin: Option<String>,
    pub plugin_opts: Option<BTreeMap<String, String>>,
}
impl SsNode {
    /// Convert a SS link to a SS node.
    /// Reference: [SS URI Scheme](https://shadowsocks.org/guide/sip002.html)
    /// ```
    /// SS-URI = "ss://" userinfo "@" hostname ":" port [ "/" ] [ "?" plugin ] [ "#" tag ]
    /// userinfo = websafe-base64-encode-utf8(method ":" password)
    ///            method ":" password
    /// ```
    pub fn from_url(url: &Url) -> Result<Self> {
        let server = url
            .host_str()
            .ok_or_else(|| anyhow!("SS link `{}` does not contain server.", url.to_string()))?
            .to_string();

        let server_port = url
            .port()
            .ok_or_else(|| anyhow!("SS link {} does not contain port.", url.to_string()))?;

        let (method, password) = if let Some(password) = url.password() {
            // if password exists, means userinfo is not encoded with Base64URL.
            let method_str = percent_decode_str(url.username()).decode_utf8_lossy();
            let method = Method::from_alias(&method_str).ok_or_else(|| {
                anyhow!(
                    "Unknown method `{}` in SS link `{}`",
                    method_str,
                    url.to_string()
                )
            })?;
            let password = percent_decode_str(password).decode_utf8_lossy().to_string();
            (method, password)
        } else {
            let encoded_userinfo = url.username();
            let decoded_userinfo_bytes = base64_url_no_pad.decode_to_vec(encoded_userinfo)?;
            let decoded_userinfo = String::from_utf8_lossy(&decoded_userinfo_bytes);
            trace!("decoded SS link userinfo: {}", &decoded_userinfo);

            let mut ss_userinfo_parts = decoded_userinfo.split(':');

            let method_str = ss_userinfo_parts
                .next()
                .ok_or_else(|| anyhow!("SS link `{}` does not contain method.", url.to_string()))?;
            let password = ss_userinfo_parts.next().ok_or_else(|| {
                anyhow!("SS link `{}` does not contain password.", url.to_string())
            })?;

            let method = Method::from_alias(method_str).ok_or_else(|| {
                anyhow!(
                    "Unknown method `{}` in SS link `{}`",
                    method_str,
                    url.to_string()
                )
            })?;

            (method, password.to_string())
        };

        if let Some(query) = url.query() {
            trace!("SS link plugin argument: {}", query);
        }
        let mut query = url.query_pairs();
        let (plugin, plugin_opts) =
            if let Some((_, plugin_arg)) = query.find(|(key, _)| key == "plugin") {
                let mut plugin_arg_parts = plugin_arg.split(';');
                let plugin = plugin_arg_parts.next().ok_or_else(|| {
                    anyhow!("SS link `{}` does not contain plugin.", url.to_string())
                })?;

                let plugin_opts = plugin_arg_parts
                    .filter_map(|plugin_arg_str| {
                        let mut plugin_arg_str_parts = plugin_arg_str.split('=');
                        let key = plugin_arg_str_parts.next()?;
                        let value = plugin_arg_str_parts.next().unwrap_or("");
                        Some((key.to_string(), value.to_string()))
                    })
                    .collect();

                (
                    Some(plugin.to_string()),
                    if plugin_arg.is_empty() {
                        None
                    } else {
                        Some(plugin_opts)
                    },
                )
            } else {
                (None, None)
            };

        let remarks = url
            .fragment()
            .map(|remarks| percent_decode_str(remarks).decode_utf8_lossy().to_string());

        Ok(Self {
            id: None,
            remarks,
            server,
            server_port,
            password,
            method,
            udp: None,
            udp_over_tcp: None,
            plugin,
            plugin_opts,
        })
    }

    pub fn plugin_opts_str(&self) -> Option<String> {
        self.plugin_opts.as_ref().map(|map| {
            map.iter()
                .map(|(key, value)| format!("{key}={value}"))
                .join(";")
        })
    }
}
impl super::GetNodeName for SsNode {
    fn get_name(&self) -> Option<&String> {
        self.remarks.as_ref()
    }

    fn get_server(&'_ self) -> &'_ String {
        &self.server
    }

    fn get_port(&self) -> u16 {
        self.server_port
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Method {
    // AEAD 2022 Ciphers
    #[serde(rename = "2022-blake3-aes-128-gcm")]
    Ss2022Blake3Aes128Gcm,
    #[serde(rename = "2022-blake3-aes-256-gcm")]
    Ss2022Blake3Aes256Gcm,
    #[serde(rename = "2022-blake3-chacha20-poly1305")]
    Ss2022Blake3Chacha20Poly1305,
    #[serde(rename = "2022-blake3-chacha8-poly1305")]
    Ss2022Blake3Chacha8Poly1305,

    // AEAD Ciphers
    #[serde(rename = "chacha20-poly1305")]
    AeadChacha20Poly1305,
    #[serde(rename = "aes-256-gcm")]
    AeadAes256Gcm,
    #[serde(rename = "aes-128-gcm")]
    AeadAes128Gcm,

    // Stream Ciphers
    #[serde(rename = "aes-128-ctr")]
    Aes128Ctr,
    #[serde(rename = "aes-192-ctr")]
    Aes192Ctr,
    #[serde(rename = "aes-256-ctr")]
    Aes256Ctr,
    #[serde(rename = "aes-128-cfb")]
    Aes128Cfb,
    #[serde(rename = "aes-192-cfb")]
    Aes192Cfb,
    #[serde(rename = "aes-256-cfb")]
    Aes256Cfb,
    #[serde(rename = "camellia-128-cfb")]
    Camellia128Cfb,
    #[serde(rename = "camellia-192-cfb")]
    Camellia192Cfb,
    #[serde(rename = "camellia-256-cfb")]
    Camellia256Cfb,
    #[serde(rename = "chacha20")]
    Chacha20,
    #[serde(rename = "chacha20-ietf")]
    Chacha20Ietf,
    #[serde(rename = "bf-cfb")]
    BfCfb,
    #[serde(rename = "salsa20")]
    Salsa20,
    #[serde(rename = "rc4-md5")]
    Rc4Md5,
}

impl Method {
    /// Get the method name using SCREAMING_SNAKE_CASE.
    #[allow(dead_code)]
    pub fn get_display_name(&self) -> &'static str {
        match self {
            Method::Ss2022Blake3Aes128Gcm => "2022_BLAKE3_AES_128_GCM",
            Method::Ss2022Blake3Aes256Gcm => "2022_BLAKE3_AES_256_GCM",
            Method::Ss2022Blake3Chacha20Poly1305 => "2022_BLAKE3_CHACHA20_POLY1305",
            Method::Ss2022Blake3Chacha8Poly1305 => "2022_BLAKE3_CHACHA8_POLY1305",

            Method::AeadChacha20Poly1305 => "AEAD_CHACHA20_POLY1305",
            Method::AeadAes256Gcm => "AEAD_AES_256_GCM",
            Method::AeadAes128Gcm => "AEAD_AES_128_GCM",

            Method::Aes128Ctr => "AES_128_CTR",
            Method::Aes192Ctr => "AES_192_CTR",
            Method::Aes256Ctr => "AES_256_CTR",
            Method::Aes128Cfb => "AES_128_CFB",
            Method::Aes192Cfb => "AES_192_CFB",
            Method::Aes256Cfb => "AES_256_CFB",
            Method::Camellia128Cfb => "CAMELLIA_128_CFB",
            Method::Camellia192Cfb => "CAMELLIA_192_CFB",
            Method::Camellia256Cfb => "CAMELLIA_256_CFB",
            Method::Chacha20 => "CHACHA20",
            Method::Chacha20Ietf => "CHACHA20_IETF",
            Method::BfCfb => "BF_CFB",
            Method::Salsa20 => "SALSA20",
            Method::Rc4Md5 => "RC4_MD5",
        }
    }

    /// Get the method name using kebab-case.
    #[allow(dead_code)]
    pub fn get_alias(&self) -> &'static str {
        match self {
            Method::Ss2022Blake3Aes128Gcm => "2022-blake3-aes-128-gcm",
            Method::Ss2022Blake3Aes256Gcm => "2022-blake3-aes-256-gcm",
            Method::Ss2022Blake3Chacha20Poly1305 => "2022-blake3-chacha20-poly1305",
            Method::Ss2022Blake3Chacha8Poly1305 => "2022-blake3-chacha8-poly1305",

            Method::AeadChacha20Poly1305 => "chacha20-poly1305",
            Method::AeadAes256Gcm => "aes-256-gcm",
            Method::AeadAes128Gcm => "aes-128-gcm",

            Method::Aes128Ctr => "aes-128-ctr",
            Method::Aes192Ctr => "aes-192-ctr",
            Method::Aes256Ctr => "aes-256-ctr",
            Method::Aes128Cfb => "aes-128-cfb",
            Method::Aes192Cfb => "aes-192-cfb",
            Method::Aes256Cfb => "aes-256-cfb",
            Method::Camellia128Cfb => "camellia-128-cfb",
            Method::Camellia192Cfb => "camellia-192-cfb",
            Method::Camellia256Cfb => "camellia-256-cfb",
            Method::Chacha20 => "chacha20",
            Method::Chacha20Ietf => "chacha20-ietf",
            Method::BfCfb => "bf-cfb",
            Method::Salsa20 => "salsa20",
            Method::Rc4Md5 => "rc4-md5",
        }
    }

    pub fn from_alias(alias: &str) -> Option<Self> {
        match alias {
            "2022-blake3-aes-128-gcm" => Some(Self::Ss2022Blake3Aes128Gcm),
            "2022-blake3-aes-256-gcm" => Some(Self::Ss2022Blake3Aes256Gcm),
            "2022-blake3-chacha20-poly1305" => Some(Self::Ss2022Blake3Chacha20Poly1305),
            "2022-blake3-chacha8-poly1305" => Some(Self::Ss2022Blake3Chacha8Poly1305),

            "chacha20-poly1305" => Some(Self::AeadChacha20Poly1305),
            "aes-256-gcm" => Some(Self::AeadAes256Gcm),
            "aes-128-gcm" => Some(Self::AeadAes128Gcm),

            "aes-128-ctr" => Some(Self::Aes128Ctr),
            "aes-192-ctr" => Some(Self::Aes192Ctr),
            "aes-256-ctr" => Some(Self::Aes256Ctr),
            "aes-128-cfb" => Some(Self::Aes128Cfb),
            "aes-192-cfb" => Some(Self::Aes192Cfb),
            "aes-256-cfb" => Some(Self::Aes256Cfb),
            "camellia-128-cfb" => Some(Self::Camellia128Cfb),
            "camellia-192-cfb" => Some(Self::Camellia192Cfb),
            "camellia-256-cfb" => Some(Self::Camellia256Cfb),
            "chacha20" => Some(Self::Chacha20),
            "chacha20-ietf" => Some(Self::Chacha20Ietf),
            "bf-cfb" => Some(Self::BfCfb),
            "salsa20" => Some(Self::Salsa20),
            "rc4-md5" => Some(Self::Rc4Md5),

            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ObfsArgs<'a> {
    pub obfs: Option<ObfsType>,
    pub host: Option<&'a str>,
    pub uri: Option<&'a str>,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ObfsType {
    Http,
    Tls,
}
impl Display for ObfsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObfsType::Http => write!(f, "http"),
            ObfsType::Tls => write!(f, "tls"),
        }
    }
}

pub fn parse_obfs_plugin_args(plugin_opts: &BTreeMap<String, String>) -> Result<ObfsArgs> {
    let obfs = if let Some(obfs) = plugin_opts.get("obfs") {
        match obfs.as_str() {
            "http" => Some(ObfsType::Http),
            "tls" => Some(ObfsType::Tls),
            _ => {
                return Err(anyhow!("Unknown obfs: `{}`", obfs));
            }
        }
    } else {
        None
    };
    let host = plugin_opts.get("obfs-host").map(|host| host.as_str());
    let uri = plugin_opts.get("obfs-uri").map(|uri| uri.as_str());

    Ok(ObfsArgs { obfs, host, uri })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ss_link_with_encoded_userinfo() {
        let link = Url::parse("ss://YWVzLTEyOC1nY206dGVzdA@192.168.100.1:8888#Example1").unwrap();
        let node = SsNode {
            id: None,
            remarks: Some(String::from("Example1")),
            server: String::from("192.168.100.1"),
            server_port: 8888,
            password: String::from("test"),
            method: Method::AeadAes128Gcm,
            udp: None,
            udp_over_tcp: None,
            plugin: None,
            plugin_opts: None,
        };
        assert_eq!(SsNode::from_url(&link).unwrap(), node);

        let link_with_plugin = Url::parse(
            "ss://cmM0LW1kNTpwYXNzd2Q@192.168.100.1:8888/?plugin=obfs-local%3Bobfs%3Dhttp#Example2",
        )
        .unwrap();
        let node_with_plugin = SsNode {
            id: None,
            remarks: Some(String::from("Example2")),
            server: String::from("192.168.100.1"),
            server_port: 8888,
            password: String::from("passwd"),
            method: Method::Rc4Md5,
            udp: None,
            udp_over_tcp: None,
            plugin: Some(String::from("obfs-local")),
            plugin_opts: Some(
                [("obfs".to_string(), "http".to_string())]
                    .into_iter()
                    .collect(),
            ),
        };
        assert_eq!(
            SsNode::from_url(&link_with_plugin).unwrap(),
            node_with_plugin
        );
    }

    #[test]
    fn parse_ss_link_with_plain_userinfo() {
        let link = Url::parse("ss://2022-blake3-aes-256-gcm:YctPZ6U7xPPcU%2Bgp3u%2B0tx%2FtRizJN9K8y%2BuKlW2qjlI%3D@192.168.100.1:8888#Example3").unwrap();
        let node = SsNode {
            id: None,
            remarks: Some(String::from("Example3")),
            server: String::from("192.168.100.1"),
            server_port: 8888,
            password: String::from("YctPZ6U7xPPcU+gp3u+0tx/tRizJN9K8y+uKlW2qjlI="),
            method: Method::Ss2022Blake3Aes256Gcm,
            udp: None,
            udp_over_tcp: None,
            plugin: None,
            plugin_opts: None,
        };
        assert_eq!(SsNode::from_url(&link).unwrap(), node);

        let link_with_plugin = Url::parse(
            "ss://2022-blake3-aes-256-gcm:YctPZ6U7xPPcU%2Bgp3u%2B0tx%2FtRizJN9K8y%2BuKlW2qjlI%3D@192.168.100.1:8888/?plugin=v2ray-plugin%3Bserver#Example4",
        )
        .unwrap();
        let node_with_plugin = SsNode {
            id: None,
            remarks: Some(String::from("Example4")),
            server: String::from("192.168.100.1"),
            server_port: 8888,
            password: String::from("YctPZ6U7xPPcU+gp3u+0tx/tRizJN9K8y+uKlW2qjlI="),
            method: Method::Ss2022Blake3Aes256Gcm,
            udp: None,
            udp_over_tcp: None,
            plugin: Some(String::from("v2ray-plugin")),
            plugin_opts: Some(
                [("server".to_string(), "".to_string())]
                    .into_iter()
                    .collect(),
            ),
        };
        assert_eq!(
            SsNode::from_url(&link_with_plugin).unwrap(),
            node_with_plugin
        );
    }
}
