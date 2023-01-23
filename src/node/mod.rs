use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

pub mod ss;
pub mod ssr;

pub use ss::SsNode;
pub use ssr::SsrNode;

#[enum_dispatch]
pub trait GetNodeName {
    fn get_name(&self) -> String;
}
impl<T> GetNodeName for Box<T>
where
    T: GetNodeName,
{
    fn get_name(&self) -> String {
        self.as_ref().get_name()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
#[enum_dispatch(GetNodeName)]
pub enum Node {
    Ss(Box<SsNode>),
    Ssr(Box<SsrNode>),
}
