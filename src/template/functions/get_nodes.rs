use std::collections::HashMap;

use serde_json::{json, Value};
use tera::{Error, Function};

use crate::template::adaptors::{
    get_adaptor_from_args, ConvertNodesToString, NodesSerializationOptions,
};
use crate::template::TemplateArgs;

use super::{get_filtered_nodes_by_function_args, RingerFunctions};

pub struct GetNodes<'a>(&'a TemplateArgs<'a>);
impl<'a> GetNodes<'a> {
    pub fn new(args: &'a TemplateArgs) -> Self {
        Self(args)
    }
}
impl<'a> RingerFunctions for GetNodes<'a> {
    const NAME: &'static str = "get_nodes";
}
impl<'a> Function for GetNodes<'a> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let nodes = get_filtered_nodes_by_function_args(Self::NAME, self.0, args)?;

        if let Some(adaptor) =
            get_adaptor_from_args(args).map_err(|err| Error::msg(err.to_string()))?
        {
            let options = NodesSerializationOptions::from_function_args(Self::NAME, args)?;
            Ok(Value::String(
                adaptor.nodes_to_string(nodes.into_iter(), options),
            ))
        } else {
            let value = nodes.collect::<Vec<_>>();
            Ok(json!(value))
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}
