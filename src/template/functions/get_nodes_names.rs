use std::collections::HashMap;

use serde_json::Value;
use tera::Function;

use crate::node::GetNodeName;
use crate::template::TemplateArgs;

use super::{get_filtered_nodes_by_function_args, RingerFunctions};

pub struct GetNodesNames<'a>(&'a TemplateArgs<'a>);
impl<'a> GetNodesNames<'a> {
    pub fn new(args: &'a TemplateArgs) -> Self {
        Self(args)
    }
}
impl<'a> RingerFunctions for GetNodesNames<'a> {
    const NAME: &'static str = "get_nodes_names";
}
impl<'a> Function for GetNodesNames<'a> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let nodes = get_filtered_nodes_by_function_args(Self::NAME, self.0, args)?;

        Ok(Value::Array(
            nodes
                .into_iter()
                .map(|node| node.get_display_name())
                .map(Value::String)
                .collect(),
        ))
    }

    fn is_safe(&self) -> bool {
        true
    }
}
