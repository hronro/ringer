use std::collections::HashMap;

use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
use log::warn;
use serde_json::Value;

use crate::node::{GetNodeName, Node};

pub mod clash;
mod clash_meta;
mod sing_box;
mod surge;

#[derive(Debug)]
pub struct NodesSerializationOptions {
    pub include_array_brackets: bool,
}
impl Default for NodesSerializationOptions {
    fn default() -> Self {
        Self {
            include_array_brackets: true,
        }
    }
}
impl NodesSerializationOptions {
    pub fn from_function_args(
        function_name: &'static str,
        args: &HashMap<String, Value>,
    ) -> std::result::Result<Self, tera::Error> {
        let mut options = Self::default();
        if let Some(include_array_brackets) = args.get("include_array_brackets") {
            if let Value::Bool(include_array_brackets) = include_array_brackets {
                options.include_array_brackets = *include_array_brackets;
            } else {
                return Err(tera::Error::msg(format!(
                    "Function `{function_name}` received an incorrect type for arg `include_array_brackets`: \
                        got `{include_array_brackets}` but expected bool",
                )));
            }
        }

        Ok(options)
    }
}

trait Adaptor: Default {
    const ADAPTOR_NAME: &'static str;

    type Node<'a>;

    fn convert_node<'a>(&self, node: &'a Node) -> Option<Self::Node<'a>>;

    fn serialize_nodes<'a, T: Iterator<Item = Self::Node<'a>>>(
        &self,
        nodes: T,
        options: NodesSerializationOptions,
    ) -> String;
}

#[enum_dispatch]
pub trait ConvertNodesToString {
    fn nodes_to_string<'a, T: Iterator<Item = &'a Node>>(
        &'_ self,
        nodes: T,
        options: NodesSerializationOptions,
    ) -> String;
}

impl<T> ConvertNodesToString for T
where
    T: Adaptor,
{
    fn nodes_to_string<'a, N: Iterator<Item = &'a Node>>(
        &'_ self,
        nodes: N,
        options: NodesSerializationOptions,
    ) -> String {
        let converted_nodes = nodes.filter_map(|node| {
            let converted_node = self.convert_node(node);
            if converted_node.is_none() {
                warn!(
                    "node `{}` is not supported in {}{}, skip it",
                    node.get_display_name(),
                    Self::ADAPTOR_NAME[0..1].to_uppercase(),
                    &Self::ADAPTOR_NAME[1..],
                );
            }

            converted_node
        });

        self.serialize_nodes(converted_nodes, options)
    }
}

#[enum_dispatch(ConvertNodesToString)]
pub enum Adaptors {
    Clash(clash::Clash),
    ClashMeta(clash_meta::ClashMeta),
    SingBox(sing_box::SingBox),
    Surge(surge::Surge),
}
impl Adaptors {
    /// Determine whether the adaptor supports the node.
    pub fn support_node(&self, node: &Node) -> bool {
        match self {
            Self::Clash(adaptor) => adaptor.convert_node(node).is_some(),
            Self::ClashMeta(adaptor) => adaptor.convert_node(node).is_some(),
            Self::SingBox(adaptor) => adaptor.convert_node(node).is_some(),
            Self::Surge(adaptor) => adaptor.convert_node(node).is_some(),
        }
    }
}

pub fn get_adaptor_from_args(args: &HashMap<String, Value>) -> Result<Option<Adaptors>> {
    if let Some(Value::String(adaptor_name_from_args)) = args.get("type") {
        match adaptor_name_from_args.as_str() {
            clash::Clash::ADAPTOR_NAME => Ok(Some(Adaptors::Clash(Default::default()))),
            clash_meta::ClashMeta::ADAPTOR_NAME => {
                Ok(Some(Adaptors::ClashMeta(Default::default())))
            }
            sing_box::SingBox::ADAPTOR_NAME => Ok(Some(Adaptors::SingBox(Default::default()))),
            surge::Surge::ADAPTOR_NAME => Ok(Some(Adaptors::Surge(Default::default()))),

            _ => Err(anyhow!(
                "Unknown adaptor name: `{}`",
                adaptor_name_from_args
            )),
        }
    } else {
        Ok(None)
    }
}
