use std::collections::{HashMap, VecDeque};
use std::fs::{create_dir_all, write};
use std::path::Path;

use anyhow::Result;
use log::{debug, error};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{json, Value};
use tera::{Context, Tera};

use crate::config::SortRules;
use crate::node::{GetNodeName, Node};
use crate::provider::{Provider, Providers};

pub mod adaptors;
mod filters;
mod functions;

use filters::RingerFilter;
use functions::RingerFunctions;

#[derive(Debug, Serialize)]
pub struct TemplateArgs<'a> {
    providers: &'a [Providers],

    nodes_by_providers: Vec<Vec<&'a Node>>,

    nodes_by_provider_names: HashMap<&'a String, Vec<&'a Node>>,

    all_nodes: Vec<&'a Node>,
}

impl<'a> TemplateArgs<'a> {
    pub fn new(
        providers: &'a [Providers],
        nodes_by_providers: &'a [Vec<Node>],
        standalone_nodes: &'a [Node],
        sort_rules: &'a SortRules,
    ) -> Self {
        let nodes_by_providers_output = nodes_by_providers
            .iter()
            .enumerate()
            .map(|(index, nodes)| {
                let provider_name = providers[index].get_name();

                let mut nodes: Vec<&Node> = nodes.iter().collect();

                nodes.par_sort_unstable_by(|a, b| {
                    let priority_a =
                        sort_rules.get_node_priority(a.get_name(), provider_name, Some(index));
                    let priority_b =
                        sort_rules.get_node_priority(b.get_name(), provider_name, Some(index));

                    if priority_a != priority_b {
                        priority_b.cmp(&priority_a)
                    } else {
                        a.get_display_name().cmp(&b.get_display_name())
                    }
                });

                nodes
            })
            .collect();

        let nodes_by_provider_names = nodes_by_providers
            .iter()
            .enumerate()
            .filter_map(|(index, nodes)| {
                if let Some(provider_name) = providers[index].get_name() {
                    let mut nodes: Vec<&Node> = nodes.iter().collect();

                    nodes.par_sort_unstable_by(|a, b| {
                        let priority_a = sort_rules.get_node_priority(
                            a.get_name(),
                            Some(provider_name),
                            Some(index),
                        );
                        let priority_b = sort_rules.get_node_priority(
                            b.get_name(),
                            Some(provider_name),
                            Some(index),
                        );

                        if priority_a != priority_b {
                            priority_b.cmp(&priority_a)
                        } else {
                            a.get_display_name().cmp(&b.get_display_name())
                        }
                    });

                    Some((provider_name, nodes))
                } else {
                    None
                }
            })
            .collect();

        let mut all_nodes_with_extra_infos: Vec<_> = nodes_by_providers
            .iter()
            .enumerate()
            .flat_map(|(index, nodes)| {
                let provider_name = providers[index].get_name();

                nodes
                    .iter()
                    .map(move |node| (node, provider_name, Some(index)))
            })
            .chain(standalone_nodes.iter().map(|node| (node, None, None)))
            .collect();

        all_nodes_with_extra_infos.par_sort_unstable_by(
            |(a_node, a_provider_name, a_provider_index),
             (b_node, b_provider_name, b_provider_index)| {
                let priority_a = sort_rules.get_node_priority(
                    a_node.get_name(),
                    *a_provider_name,
                    *a_provider_index,
                );
                let priority_b = sort_rules.get_node_priority(
                    b_node.get_name(),
                    *b_provider_name,
                    *b_provider_index,
                );

                if priority_a != priority_b {
                    priority_b.cmp(&priority_a)
                } else {
                    a_node.get_display_name().cmp(&b_node.get_display_name())
                }
            },
        );

        let all_nodes = all_nodes_with_extra_infos
            .into_iter()
            .map(|(node, _, _)| node)
            .collect();

        Self {
            providers,
            nodes_by_providers: nodes_by_providers_output,
            nodes_by_provider_names,
            all_nodes,
        }
    }
}

/// Template.
/// Check [`crate::config::ConfigFileTemplate`] as a reference.
#[derive(Clone)]
pub struct Template {
    /// The name of the template.
    pub name: Option<String>,

    pub file_name: String,

    /// The template content.
    pub template: String,

    /// Specify the template must be rendered after some other templates are rendered.
    /// Then people can use `{{ output.<NAME_OF_THE_REQUIRED_TEMPLATE> }}` to inject the content
    /// of the required template into the template.
    /// Only templates with a name can be required.
    pub requires: Vec<String>,

    /// The sub-directories of output path.
    pub output_sub_directories: Vec<String>,
}
impl std::fmt::Debug for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template")
            .field("name", &self.name)
            .field("template", &"[[**TEMPLATE**]]")
            .field("requires", &self.requires)
            .field("output_sub_directories", &self.output_sub_directories)
            .finish()
    }
}

pub fn get_built_in_templates() -> Vec<Template> {
    vec![
        Template {
            name: Some(String::from("built_in_clash")),
            file_name: String::from("config.yaml"),
            template: String::from(include_str!("./built_in_templates/clash/config.yaml")),
            requires: vec![],
            output_sub_directories: vec![String::from("clash")],
        },
        Template {
            name: Some(String::from("built_in_sing_box")),
            file_name: String::from("config.json"),
            template: String::from(include_str!("./built_in_templates/sing-box/config.json")),
            requires: vec![],
            output_sub_directories: vec![String::from("sing-box")],
        },
        Template {
            name: Some(String::from("built_in_surge")),
            file_name: String::from("surge.conf"),
            template: String::from(include_str!("./built_in_templates/surge/surge.conf")),
            requires: vec![],
            output_sub_directories: vec![String::from("surge")],
        },
    ]
}

pub struct RenderEngine<'a> {
    templates: &'a [Template],
    context: Context,
    tera: Tera,
}

impl<'a> RenderEngine<'a> {
    pub fn new(args: &'static TemplateArgs<'a>, templates: &'a [Template]) -> Self {
        let context = Context::new();

        let mut tera = Tera::default();
        tera.register_filter(filters::InsertIndents::NAME, filters::InsertIndents);
        tera.register_function(functions::GetNodes::NAME, functions::GetNodes::new(args));
        tera.register_function(
            functions::GetNodesNames::NAME,
            functions::GetNodesNames::new(args),
        );

        Self {
            templates,
            context,
            tera,
        }
    }

    pub fn render<T>(&mut self, output_directory: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        let mut templates = VecDeque::from(self.templates.to_vec());

        while let Some(template) = templates.pop_front() {
            // check if all the required templates are rendered.
            if !template.requires.is_empty() {
                let ok = template.requires.iter().all(|required_template_name| {
                    if let Some(output_in_context) = self.context.get("output") {
                        output_in_context.get(required_template_name).is_some()
                    } else {
                        false
                    }
                });

                if !ok {
                    templates.push_back(template);
                    continue;
                }
            }

            let output = if let Some(template_name) = &template.name {
                self.tera
                    .add_raw_template(template_name, template.template.as_str())?;
                let output = self.tera.render(template_name, &self.context);
                if output.is_err() {
                    error!("failed to render template {}", template_name);
                }
                let output = output?;

                let output_in_context = self.context.remove("output").unwrap_or_else(|| json!({}));
                assert!(output_in_context.is_object());
                if let Value::Object(mut map) = output_in_context {
                    map.insert(template_name.to_string(), Value::String(output.clone()));
                    self.context.insert("output", &Value::Object(map));
                } else {
                    unreachable!();
                }

                output
            } else {
                let output = self
                    .tera
                    .render_str(template.template.as_str(), &self.context);
                if output.is_err() {
                    error!("failed to render {:?}", template);
                }
                output?
            };

            let output_dir = {
                let mut output_dir = output_directory.as_ref().to_path_buf();

                for sub_dir in &template.output_sub_directories {
                    output_dir.push(sub_dir);
                }

                output_dir
            };

            create_dir_all(&output_dir)?;

            let output_path = {
                let mut output_path = output_dir;

                output_path.push(&template.file_name);

                output_path
            };
            debug!("the output path of {:?} is {:?}", &template, &output_path);
            write(output_path, output)?;
        }

        Ok(())
    }
}
