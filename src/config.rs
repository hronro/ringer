use std::path::PathBuf;

use anyhow::{anyhow, Result};
use log::Level as LogLevel;
use serde::Deserialize;
use url::Url;

use crate::node::Node;
use crate::provider::{Provider, Providers};
use crate::template::Template;
use crate::utils::{load_content_from_url, parse_string_to_path, Path};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub provider: Option<ConfigFileProviderOrProviders>,

    pub node: Option<ConfigFileNodeOrNodes>,

    pub template: Option<ConfigFileTemplateOrTemplates>,
}

impl ConfigFile {
    pub fn rewrite_relative_path(&mut self, config_file_path: Path) -> Result<()> {
        match config_file_path {
            Path::Url(url) => {
                let url = Url::parse(url.to_string().as_str()).unwrap();

                if let Some(providers) = &self.provider {
                    let mut providers = providers.clone();

                    match providers {
                        ConfigFileProviderOrProviders::Provider(ref mut p) => {
                            let new_url = url.join(p.get_url().to_string().as_str())?;
                            p.set_url(new_url.as_str().parse().unwrap());
                        }
                        ConfigFileProviderOrProviders::Providers(ref mut ps) => {
                            for ref mut p in ps {
                                let new_url = url.join(p.get_url().to_string().as_str())?;
                                p.set_url(new_url.as_str().parse().unwrap());
                            }
                        }
                    }

                    self.provider = Some(providers);
                }

                if let Some(templates) = &self.template {
                    let mut templates = templates.clone();

                    match templates {
                        ConfigFileTemplateOrTemplates::Template(ref mut t) => {
                            let new_url = url.join(t.path.to_string().as_str())?;
                            t.path = new_url.as_str().parse().unwrap();
                        }
                        ConfigFileTemplateOrTemplates::Templates(ref mut ts) => {
                            for ref mut t in ts {
                                let new_url = url.join(t.path.to_string().as_str())?;
                                t.path = new_url.as_str().parse().unwrap();
                            }
                        }
                    }

                    self.template = Some(templates);
                }
            }

            Path::PathBuf(path_buf) => {
                if let Some(templates) = &self.template {
                    let mut templates = templates.clone();

                    match templates {
                        ConfigFileTemplateOrTemplates::Template(ref mut t) => {
                            let t_path = parse_string_to_path(t.path.clone())?;
                            if let Path::PathBuf(t_path) = t_path {
                                let mut new_path = path_buf.parent().unwrap().to_path_buf();
                                new_path.push(t_path);
                                t.path = new_path.to_string_lossy().to_string();
                            };
                        }
                        ConfigFileTemplateOrTemplates::Templates(ref mut ts) => {
                            for ref mut t in ts {
                                let t_path = parse_string_to_path(t.path.clone())?;
                                if let Path::PathBuf(t_path) = t_path {
                                    let mut new_path = path_buf.parent().unwrap().to_path_buf();
                                    new_path.push(t_path);
                                    t.path = new_path.to_string_lossy().to_string();
                                };
                            }
                        }
                    }

                    self.template = Some(templates);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ConfigFileProviderOrProviders {
    Provider(Providers),
    Providers(Vec<Providers>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ConfigFileNodeOrNodes {
    Node(Node),
    Nodes(Vec<Node>),
}

/// Template definition used in the config file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFileTemplate {
    /// The name of the template.
    pub name: Option<String>,

    /// The file name of the output.
    /// If not specified, the output file name will be inferred from the template path.
    pub file_name: Option<String>,

    /// The template path.
    pub path: String,

    /// Specify the template must be rendered after some other templates are rendered.
    /// Then people can use `{{ output.<NAME_OF_THE_REQUIRED_TEMPLATE> }}` to inject the content
    /// of the required template into the template.
    /// Only templates with a name can be required.
    pub requires: Option<Vec<String>>,

    /// The sub-directories of output path.
    /// By default will save the file to in the directory specified CLI arguments,
    /// but if you want to save to file to `<OUTPUT_PATH_IN_CLI>/foo/bar/<file>`,
    /// you can specify `output_sub_directories` to `["foo", "bar"]`.
    pub output_sub_directories: Option<Vec<String>>,
}

impl ConfigFileTemplate {
    pub async fn into_tempalte(self) -> Result<Template> {
        let file_name = self.file_name.map(Ok).unwrap_or_else(|| {
            self.path
                .split('/')
                .last()
                .map(String::from)
                .ok_or_else(|| {
                    anyhow!(
                        "can not infer template file name from path `{}`",
                        self.path.to_string()
                    )
                })
        })?;

        let path = parse_string_to_path(self.path)?;

        let content = load_content_from_url(path).await?;

        Ok(Template {
            name: self.name,
            file_name,
            template: String::from_utf8_lossy(&content).to_string(),
            requires: self.requires.unwrap_or_default(),
            output_sub_directories: self.output_sub_directories.unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ConfigFileTemplateOrTemplates {
    Template(ConfigFileTemplate),
    Templates(Vec<ConfigFileTemplate>),
}

/// Load a config file from an URL.
pub async fn load_config_file(path: Path) -> Result<ConfigFile> {
    let contents = load_content_from_url(path).await?;
    Ok(toml::from_slice(&contents)?)
}

/// The final config merged from CLI arguments and config file.
#[derive(Debug)]
pub struct MergedConfig {
    pub providers: Vec<Providers>,

    pub standalone_nodes: Vec<Node>,

    pub templates: Vec<Template>,

    pub output_directory: PathBuf,

    pub log_level: LogLevel,
}
