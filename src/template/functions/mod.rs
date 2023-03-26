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
    args: &'a HashMap<String, Value>,
) -> Result<Box<dyn Iterator<Item = &'a Node> + 'a>, Error> {
    if args.get("provider").is_some() && args.get("provider_index").is_some() {
        return Err(Error::msg(format!("Function `{function_name}` received two args (`provider` and `provider_index`) that conflict with each other. Please choose one of them or none of them.")));
    }

    let nodes: Box<dyn Iterator<Item = &'a Node>> = if let Some(provider_name) =
        args.get("provider")
    {
        if let Value::String(provider_name) = provider_name {
            if let Some(nodes) = template_args.nodes_by_provider_names.get(provider_name) {
                Box::new(nodes.iter().copied())
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider`: \
                        get `{provider_name}` but provider with name `{provider_name}` doesn't exists",
                )));
            }
        } else if let Value::Array(provider_name_array) = provider_name {
            if provider_name_array.is_empty() {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider`: \
                        received an array but the array is empty",
                )));
            }

            if provider_name_array.iter().all(|value| value.is_string()) {
                let mut nodes: Box<dyn Iterator<Item = &'a Node>> = Box::new([].into_iter());
                for provider_name_value in provider_name_array {
                    if let Value::String(provider_name) = provider_name_value {
                        if let Some(provider_nodes) =
                            template_args.nodes_by_provider_names.get(provider_name)
                        {
                            nodes = Box::new(nodes.chain(provider_nodes.iter().copied()));
                        } else {
                            return Err(Error::msg(format!(
                                "Function `{function_name}` received an incorrect value for arg `provider`: \
                                    get `{provider_name}` in the array but provider with name `{provider_name}` doesn't exists",
                            )));
                        }
                    } else {
                        unreachable!()
                    }
                }
                nodes
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect type for arg `provider`: \
                        get `{provider_name}` but expected String or Array of Strings",
                )));
            }
        } else {
            return Err(Error::msg(format!(
                "Function `{function_name}` received an incorrect type for arg `provider`: \
                    get `{provider_name}` but expected String or Array of Strings",
            )));
        }
    } else if let Some(provider_index) = args.get("provider_index") {
        if let Some(provider_index) = provider_index.as_u64() {
            if let Some(nodes) = template_args
                .nodes_by_providers
                .get(provider_index as usize)
            {
                Box::new(nodes.iter().copied())
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider_index`: \
                        get `{provider_index}` but provider with index `{provider_index}` doesn't exists",
                )));
            }
        } else if let Value::Array(provider_index_array) = provider_index {
            if provider_index_array.is_empty() {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect value for arg `provider_index`: \
                        received an array but the array is empty",
                )));
            }

            if provider_index_array.iter().all(|value| value.is_u64()) {
                let mut nodes: Box<dyn Iterator<Item = &'a Node>> = Box::new([].into_iter());
                for provider_index_value in provider_index_array {
                    if let Some(provider_index) = provider_index_value.as_u64() {
                        if let Some(provider_nodes) = template_args
                            .nodes_by_providers
                            .get(provider_index as usize)
                        {
                            nodes = Box::new(nodes.chain(provider_nodes.iter().copied()))
                        } else {
                            return Err(Error::msg(format!(
                                "Function `{function_name}` received an incorrect value for arg `provider_index`: \
                                    get `{provider_index}` in the array but provider with index `{provider_index}` doesn't exists",
                            )));
                        }
                    } else {
                        unreachable!()
                    }
                }
                nodes
            } else {
                return Err(Error::msg(format!(
                    "Function `{function_name}` received an incorrect type for arg `provider_index`: \
                        get `{provider_index}` but expected Integer or Array of Integers",
                )));
            }
        } else {
            return Err(Error::msg(format!(
                "Function `{function_name}` received an incorrect type for arg `provider_index`: \
                    get `{provider_index}` but expected Integer or Array of Integers",
            )));
        }
    } else {
        Box::new(template_args.all_nodes.iter().copied())
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
        Ok(nodes)
    } else {
        Ok(Box::new(nodes.filter(move |node| {
            if let Some(name) = node.get_name() {
                if let Some(name_not_starts_with) = name_not_starts_with {
                    if name.starts_with(name_not_starts_with) {
                        return false;
                    }
                }

                if let Some(name_not_ends_with) = name_not_ends_with {
                    if name.ends_with(name_not_ends_with) {
                        return false;
                    }
                }

                if let Some(name_not_contains) = name_not_contains {
                    if name.contains(name_not_contains) {
                        return false;
                    }
                }

                if let Some(name_starts_with) = name_starts_with {
                    if !name.starts_with(name_starts_with) {
                        return false;
                    }
                }

                if let Some(name_ends_with) = name_ends_with {
                    if !name.ends_with(name_ends_with) {
                        return false;
                    }
                }

                if let Some(name_contains) = name_contains {
                    if !name.contains(name_contains) {
                        return false;
                    }
                }

                true
            } else {
                false
            }
        })))
    }
}
