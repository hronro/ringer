use std::collections::HashMap;

use serde_json::Value;
use tera::{Error, Function};

use crate::node::GetNodeName;
use crate::template::adaptors::get_adaptor_from_args;
use crate::template::TemplateArgs;

use super::{get_filtered_nodes_by_function_args, RingerFunctions};

pub struct GetNodesNames<'a>(&'a TemplateArgs<'a>);
impl<'a> GetNodesNames<'a> {
    pub fn new(args: &'a TemplateArgs) -> Self {
        Self(args)
    }
}
impl RingerFunctions for GetNodesNames<'_> {
    const NAME: &'static str = "get_nodes_names";
}
impl Function for GetNodesNames<'_> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let nodes = get_filtered_nodes_by_function_args(Self::NAME, self.0, args)?;

        if let Some(adaptor) =
            get_adaptor_from_args(args).map_err(|err| Error::msg(err.to_string()))?
        {
            Ok(Value::Array(
                nodes
                    .filter(|node| adaptor.support_node(node))
                    .map(|node| Value::String(node.get_display_name()))
                    .collect(),
            ))
        } else {
            Ok(Value::Array(
                nodes
                    .map(|node| Value::String(node.get_display_name()))
                    .collect(),
            ))
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}
