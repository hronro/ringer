use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use base64_simd::URL_SAFE_NO_PAD as base64_url_no_pad;
use log::trace;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SsrNode {
    pub remarks: Option<String>,
    pub server: String,
    pub server_port: u16,
    pub password: String,
    pub method: String,
    pub protocol: String,
    pub protocol_param: Option<String>,
    pub obfs: String,
    pub obfs_param: Option<String>,
    pub udpport: Option<u16>,
    pub uot: Option<bool>,
}
impl SsrNode {
    /// Convert a SSR link to a SSR node.
    /// Reference: [SSR link format](https://github.com/shadowsocksr-backup/shadowsocks-rss/wiki/SSR-QRcode-scheme)
    /// ```
    /// ssr://base64(host:port:protocol:method:obfs:base64pass/?obfsparam=base64param&protoparam=base64param&remarks=base64remarks&group=base64group&udpport=0&uot=0)
    /// ```
    pub fn from_url(url: &Url) -> Result<Self> {
        trace!("SSR link: {}", url.to_string());
        if let Some(encoded_content) = url.host() {
            let decoded_host_bytes = base64_url_no_pad
                .decode_to_vec(encoded_content.to_string())
                .context("failed to decode base64 for the SSR link")?;
            let decoded_host = String::from_utf8_lossy(&decoded_host_bytes);
            trace!("decoded SSR link: {}", &decoded_host);

            let mut ssr_url_parts = decoded_host.split("/?");
            let ssr_url_part1 = ssr_url_parts.next().unwrap();
            let ssr_url_part2 = ssr_url_parts.next();

            let part1_parts = ssr_url_part1.split(':').collect::<Vec<&str>>();
            let (server, server_port, protocol, method, obfs, password) = (
                part1_parts[0].to_string(),
                part1_parts[1]
                    .parse::<u16>()
                    .context("failed to parse server port")?,
                part1_parts[2].to_string(),
                part1_parts[3].to_string(),
                part1_parts[4].to_string(),
                String::from_utf8_lossy(
                    &base64_url_no_pad
                        .decode_to_vec(part1_parts[5])
                        .context("failed to decode base64 for the password")?,
                )
                .to_string(),
            );

            let (remarks, obfs_param, protocol_param, udpport, uot) =
                if let Some(ssr_url_part2) = ssr_url_part2 {
                    let query: HashMap<String, String> = serde_urlencoded::from_str(ssr_url_part2)
                        .context("failed to parse qeury")?;

                    (
                        query.get("remarks").and_then(|base64remarks| {
                            let value = String::from_utf8_lossy(
                                &base64_url_no_pad.decode_to_vec(base64remarks).unwrap(),
                            )
                            .to_string();
                            if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            }
                        }),
                        query.get("obfsparam").and_then(|base64param| {
                            let value = String::from_utf8_lossy(
                                &base64_url_no_pad.decode_to_vec(base64param).unwrap(),
                            )
                            .to_string();
                            if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            }
                        }),
                        query.get("protoparam").and_then(|base64param| {
                            let value = String::from_utf8_lossy(
                                &base64_url_no_pad.decode_to_vec(base64param).unwrap(),
                            )
                            .to_string();
                            if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            }
                        }),
                        query
                            .get("udpport")
                            .map(|udpport_string| udpport_string.parse::<u16>().unwrap()),
                        query
                            .get("uot")
                            .map(|uot_string| uot_string.parse::<u8>().unwrap() != 0),
                    )
                } else {
                    (None, None, None, None, None)
                };

            Ok(Self {
                remarks,
                server,
                server_port,
                password,
                method,
                protocol,
                protocol_param,
                obfs,
                obfs_param,
                udpport,
                uot,
            })
        } else {
            Err(anyhow!("empty host for the link `{}`", url))
        }
    }
}
impl super::GetNodeName for SsrNode {
    fn get_display_name(&self) -> String {
        if let Some(remarks) = self.remarks.as_ref() {
            remarks.clone()
        } else {
            format!("{}:{}", &self.server, self.server_port)
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssr_link() {
        let link = Url::parse("ssr://MTI3LjAuMC4xOjEyMzQ6YXV0aF9hZXMxMjhfbWQ1OmFlcy0xMjgtY2ZiOnRsczEuMl90aWNrZXRfYXV0aDpZV0ZoWW1KaS8_b2Jmc3BhcmFtPVluSmxZV3QzWVRFeExtMXZaUSZyZW1hcmtzPTVyV0w2Sy1WNUxpdDVwYUg").unwrap();

        let node = SsrNode {
            remarks: Some(String::from("测试中文")),
            server: String::from("127.0.0.1"),
            server_port: 1234,
            password: String::from("aaabbb"),
            method: String::from("aes-128-cfb"),
            protocol: String::from("auth_aes128_md5"),
            protocol_param: None,
            obfs: String::from("tls1.2_ticket_auth"),
            obfs_param: Some(String::from("breakwa11.moe")),
            udpport: None,
            uot: None,
        };

        assert_eq!(SsrNode::from_url(&link).unwrap(), node);
    }

    #[test]
    fn parse_ssr_link_without_remarks() {
        let link = Url::parse("ssr://MTI3LjAuMC4xOjEyMzQ6YXV0aF9hZXMxMjhfbWQ1OmFlcy0xMjgtY2ZiOnRsczEuMl90aWNrZXRfYXV0aDpZV0ZoWW1KaS8_b2Jmc3BhcmFtPVluSmxZV3QzWVRFeExtMXZaUQ").unwrap();

        let node = SsrNode {
            remarks: None,
            server: String::from("127.0.0.1"),
            server_port: 1234,
            password: String::from("aaabbb"),
            method: String::from("aes-128-cfb"),
            protocol: String::from("auth_aes128_md5"),
            protocol_param: None,
            obfs: String::from("tls1.2_ticket_auth"),
            obfs_param: Some(String::from("breakwa11.moe")),
            udpport: None,
            uot: None,
        };

        assert_eq!(SsrNode::from_url(&link).unwrap(), node);
    }
}
