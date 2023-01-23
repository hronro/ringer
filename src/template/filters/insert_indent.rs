use const_format::formatcp;
use serde_json::Value;
use tera::{Error, Filter};

use super::RingerFilter;

pub struct InsertIndent;
impl RingerFilter for InsertIndent {
    const NAME: &'static str = "insert_indent";
}
impl Filter for InsertIndent {
    fn filter(
        &self,
        value: &Value,
        args: &std::collections::HashMap<String, Value>,
    ) -> tera::Result<Value> {
        if let Value::String(input_string) = value {
            let indents: String = if let Some(tabs) = args.get("tabs") {
                if let Some(tabs) = tabs.as_u64() {
                    Ok(vec!['\t'; tabs as usize].into_iter().collect())
                } else {
                    Err(Error::msg(format!(
                        "Filter `{}` received an incorrect type for arg `tabs`: \
                             got `{}` but expected u64",
                        Self::NAME,
                        value
                    )))
                }
            } else if let Some(spaces) = args.get("spaces") {
                if let Some(spaces) = spaces.as_u64() {
                    Ok(vec![' '; spaces as usize].into_iter().collect())
                } else {
                    Err(Error::msg(format!(
                        "Filter `{}` received an incorrect type for arg `spaces`: \
                             got `{}` but expected u64",
                        Self::NAME,
                        value
                    )))
                }
            } else {
                // Default to 2 spaces.
                Ok(String::from("  "))
            }?;

            if indents.is_empty() {
                return Ok(value.clone());
            }

            let include_first_line =
                if let Some(include_first_line) = args.get("include_first_line") {
                    if let Some(include_first_line) = include_first_line.as_bool() {
                        Ok(include_first_line)
                    } else {
                        Err(Error::msg(format!(
                            "Filter `{}` received an incorrect type for arg `include_first_line`: \
                                 got `{}` but expected bool",
                            Self::NAME,
                            value
                        )))
                    }
                } else {
                    Ok(false)
                }?;

            let output_string = input_string
                .split('\n')
                .enumerate()
                .map(|(index, line)| {
                    if index == 0 && !include_first_line {
                        line.to_string()
                    } else {
                        format!("{indents}{line}")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");

            Ok(Value::String(output_string))
        } else {
            Err(Error::msg(formatcp!(
                "Filter `{}` was used on a value that isn't a string.",
                InsertIndent::NAME
            )))
        }
    }
}
