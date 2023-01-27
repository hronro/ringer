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

    macro_rules! create_string_arg {
        ($arg_name: ident) => {
            let $arg_name = if let Some(value) = args.get(stringify!($arg_name)) {
                if let Value::String($arg_name) = value {
                    Some($arg_name)
                } else {
                    return Err(Error::msg(format!(
                        "Function `{function_name}` received an incorrect type for arg `{}`: \
                            get `{value}` but expected String",
                        stringify!($arg_name),
                    )));
                }
            } else {
                None
            };
        };
    }

    create_string_arg!(name_starts_with);
    create_string_arg!(name_not_starts_with);
    create_string_arg!(name_ends_with);
    create_string_arg!(name_not_ends_with);
    create_string_arg!(name_contains);
    create_string_arg!(name_not_contains);

    if name_starts_with.is_none()
        && name_not_starts_with.is_none()
        && name_ends_with.is_none()
        && name_not_ends_with.is_none()
        && name_contains.is_none()
        && name_not_contains.is_none()
    {
        Ok(nodes.clone())
    } else {
        Ok(nodes
            .iter()
            .filter_map(|node| {
                if let Some(name) = node.get_name() {
                    if let Some(name_not_starts_with) = name_not_starts_with {
                        if name.starts_with(name_not_starts_with) {
                            return None;
                        }
                    }

                    if let Some(name_not_ends_with) = name_not_ends_with {
                        if name.ends_with(name_not_ends_with) {
                            return None;
                        }
                    }

                    if let Some(name_not_contains) = name_not_contains {
                        if name.contains(name_not_contains) {
                            return None;
                        }
                    }

                    if let Some(name_starts_with) = name_starts_with {
                        if !name.starts_with(name_starts_with) {
                            return None;
                        }
                    }

                    if let Some(name_ends_with) = name_ends_with {
                        if !name.ends_with(name_ends_with) {
                            return None;
                        }
                    }

                    if let Some(name_contains) = name_contains {
                        if !name.contains(name_contains) {
                            return None;
                        }
                    }

                    Some(*node)
                } else {
                    None
                }
            })
            .collect())
    }
}
