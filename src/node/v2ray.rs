use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VMessNode {
    pub tag: Option<String>,
    pub address: String,
    pub port: u16,
    pub uuid: Uuid,
    pub transport: Option<transport::Transport>,
}
impl super::GetNodeName for VMessNode {
    fn get_display_name(&self) -> String {
        if let Some(tag) = self.tag.as_ref() {
            tag.clone()
        } else {
            format!("{}:{}", &self.address, self.port)
        }
    }

    fn get_name(&self) -> Option<&String> {
        self.tag.as_ref()
    }

    fn get_server(&'_ self) -> &'_ String {
        &self.address
    }

    fn get_port(&self) -> u16 {
        self.port
    }
}

/// 
pub mod transport {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Transport {
        MKcp(MKcpSettings),
        Tcp,
        WebSocket,
        GRpc(GRpcSettings),
        Quic,
    }

    /// Settings of mKCP
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct MKcpSettings {
        /// Maximum transmission unit.
        /// This value is typically between `576` - `1460`.
        /// It is `1350` by default.
        pub mtu: Option<u16>,

        /// Transmission time interval in a millisecond.
        /// mKCP will send data at this frequency.
        /// Please choose a value between `10` - `100`.
        /// It is `50` by default.
        pub tti: Option<u8>,

        /// Upload bandwidth capacity.
        /// The maximum speed to send data in MB/s.
        /// It is `5` by default.
        /// Beware it is Byte, not Bit.
        /// You can set it to `0` for very low bandwidth.
        pub uplink_capacity: Option<u32>,

        /// Download bandwidth capacity.
        /// The maximum speed to receive data in MB/s.
        /// It is `20` by default.
        /// Beware it is Byte, not Bit.
        /// You can set it to `0` for very low bandwidth.
        pub downlink_capacity: Option<u32>,

        /// Whether congestion control is enabled.
        /// It is `false` by default.
        /// This will instruct V2Ray to decrease transfer speed if there is too much packet loss.
        pub congestion: Option<bool>,

        /// The read buffer size of a single connection, in MB.
        /// It is `2` by default.
        pub read_buffer_size: Option<u32>,

        /// The write buffer size of a single connection, in MB.
        /// It is `2` by default.
        pub write_buffer_size: Option<u32>,

        /// The encryption seed for traffic obfuscator. Need to be the same on both sides.
        pub seed: Option<String>,
    }

    /// Settings of WebSocket
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct WebSocketSettings {
        /// The HTTP path for the websocket request.
        /// Empty value means root(`"/"`).
        pub path: Option<String>,

        /// The header to be sent in HTTP request.
        pub headers: Option<HashMap<String, String>>,

        /// The max number of bytes of early data.
        pub max_early_data: Option<u32>,

        /// Whether to enable browser forwarder.
        pub use_browser_forwarding: Option<bool>,

        /// The header name for WebSocket Early Data.
        /// If not set, the early data will be send through path.
        pub early_data_header_name: Option<String>,
    }

    /// Settings of gRPC
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GRpcSettings {
        /// Name of the gRPC service.
        pub stream: Option<String>,
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Security {}
