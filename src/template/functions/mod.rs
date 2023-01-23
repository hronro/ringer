use std::collections::HashMap;

use serde_json::Value;
use tera::Error;

use crate::node::{GetNodeName, Node};
use crate::template::TemplateArgs;

mod get_nodes;
mod get_nodes_names;

pub use get_nodes::GetNodes;
pub use get_nodes_names::GetNodesNames;

pub trait RingerFunctions {
    const NAME: &'static str;
}

fn get_filtered_nodes_by_function_args<'a>(
    function_name: &'static str,
    template_args: &'a TemplateArgs<'a>,
    args: &HashMap<String, Value>,
) -> Result<Vec<&'a Node>, Error> {
    if args.get("provider").is_some() && args.get("provider_index").is_some() {
        return Err(Error::msg(format!("Function `{function_name}` received two args (`provider` and `provider_index`) that conflict with each other. Please choose one of them or none of them.")));
    }

    let nodes = if let Some(provider_name) = args.get("provider") {
        if let Value::String(provider_name) = provider_name {
            if let Some(nodes) = template_args.nodes_by_provider_names.get(provider_name) {
                nodes
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider`: \
                        get `{provider_name}` but provider with name `{provider_name}` doesn't exists",
                )));
            }
        } else {
            return Err(Error::msg(format!(
                "Function `{function_name}` received an incorrect type for arg `provider`: \
                    get `{provider_name}` but expected String",
            )));
        }
    } else if let Some(provider_index) = args.get("provider_index") {
        if let Some(provider_index) = provider_index.as_u64() {
            if let Some(nodes) = template_args
                .nodes_by_providers
                .get(provider_index as usize)
            {
                nodes
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider_index`: \
                        get `{provider_index}` but provider with index `{provider_index}` doesn't exists",
                )));
            }
        } else {
            return Err(Error::msg(format!(
                "Function `{function_name}` received an incorrect type for arg `provider_index`: \
                    get `{provider_index}` but expected u64",
            )));
        }
    } else {
        &template_args.all_nodes
    };

    if let Some(name_contains) = args.get("name_contains") {
        if let Value::String(name_contains) = name_contains {
            Ok(nodes
                .iter()
                .filter_map(|node| {
                    if let Some(name) = node.get_name() {
                        if name.contains(name_contains) {
                            Some(*node)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect())
        } else {
            Err(Error::msg(format!(
                "Function `{function_name}` received an incorrect type for arg `name_contains`: \
                    get `{name_contains}` but expected String",
            )))
        }
    } else {
        Ok(nodes.clone())
    }
}
