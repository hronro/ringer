use std::collections::HashMap;

use serde_json::Value;
use tera::Function;

use crate::node::GetNodeName;
use crate::template::TemplateArgs;

pub struct GetNodesNames<'a>(&'a TemplateArgs<'a>);
impl<'a> GetNodesNames<'a> {
    pub fn new(args: &'a TemplateArgs) -> Self {
        Self(args)
    }
}
impl<'a> super::RingerFunctions for GetNodesNames<'a> {
    const NAME: &'static str = "get_nodes_names";
}
impl<'a> Function for GetNodesNames<'a> {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        Ok(Value::Array(
            self.0
                .all_nodes
                .iter()
                .map(|node| node.get_name())
                .map(Value::String)
                .collect(),
        ))
    }

    fn is_safe(&self) -> bool {
        true
    }
}
