use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsOptions {
    pub sni: Option<String>,
    pub insecure: Option<bool>,
    pub alpn: Option<Vec<String>>,
}
