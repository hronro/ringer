use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{ArgAction, Parser, ValueEnum};
use futures::future::try_join_all;
use http::Uri;
use log::{debug, Level as LogLevel};
use url::Url;

use crate::config::{
    load_config_file, ConfigFileNodeOrNodes, ConfigFileProviderOrProviders,
    ConfigFileSortRuleOrSortRules, ConfigFileTemplate, ConfigFileTemplateOrTemplates, MergedConfig,
    SortRules,
};
use crate::provider::{Clash, Providers, Ssr};
use crate::template::get_built_in_templates;
use crate::utils::parse_string_to_path;

#[derive(Debug, ValueEnum, Clone)]
pub enum CliProviderType {
    Ssr,
    Clash,
}

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    /// The path of a custom config file.
    #[arg(short, long)]
    config: Option<String>,

    /// Set the provider type. Could be used multiple times for setting multiple providers.
    #[arg(short = 'p', long)]
    provider_type: Vec<CliProviderType>,

    /// Set the provider URL. Could be used multiple times for setting multiple providers.
    #[arg(short = 'u', long)]
    provider_url: Vec<Url>,

    /// Whether using the built-in templates.
    /// If not set, will automatically set to `true` when there are no user-specified templates,
    /// otherwise it will be set to `false`.
    /// You can disable this behavior by explicitly specifying this option.
    #[arg(short = 'b', long = "built-in-templates")]
    use_built_in_templates: Option<bool>,

    /// Set template path. Could be used multiple times for setting multiple templates.
    #[arg(short = 't', long)]
    template: Vec<String>,

    /// The path of the output directory.
    #[arg(short = 'o', long, value_name = "PATH")]
    output_directory: Option<PathBuf>,

    /// A level of verbosity, and can be used multiple times
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

/// Get the final config from both the CLI arguments and config file.
/// CLI arguments have higher priority than the config file.
pub async fn get_config() -> Result<MergedConfig> {
    let cli_config = Opts::parse();

    if cli_config.provider_type.len() != cli_config.provider_url.len() {
        return Err(anyhow!(
            "The length of `provider_type` and `provider_url` are not equal."
        ));
    }

    let providers_from_cli: Vec<Providers> = cli_config
        .provider_type
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let url = cli_config.provider_url[index].as_str().parse().unwrap();
            get_provider_from_cli_input(ty, url)
        })
        .collect();

    let mut config_file_templates_from_cli: Vec<_> = cli_config
        .template
        .into_iter()
        .map(|template_url| ConfigFileTemplate {
            name: None,
            file_name: None,
            path: template_url,
            requires: None,
            output_sub_directories: None,
        })
        .collect();

    let (providers, standalone_nodes, sort_rules, config_file_templates, output_directory) =
        if let Some(config_file_path_string) = cli_config.config {
            let config_file_path = parse_string_to_path(config_file_path_string)
                .context("failed to parse config file path")?;
            let config_file = {
                let mut config_file = load_config_file(config_file_path.clone())
                    .await
                    .context("failed to load config file")?;
                config_file
                    .rewrite_relative_path(config_file_path)
                    .context("failed to rewrite relative path in the config file")?;
                config_file
            };

            let providers = if let Some(providers_from_config_file) = config_file.provider {
                match providers_from_config_file {
                    ConfigFileProviderOrProviders::Provider(p) => {
                        let providers_from_config_file = [p];
                        providers_from_config_file
                            .into_iter()
                            .chain(providers_from_cli)
                            .collect()
                    }
                    ConfigFileProviderOrProviders::Providers(ps) => {
                        ps.into_iter().chain(providers_from_cli).collect()
                    }
                }
            } else {
                providers_from_cli
            };

            let standalone_nodes = if let Some(nodes_from_config_file) = config_file.node {
                match nodes_from_config_file {
                    ConfigFileNodeOrNodes::Node(node) => vec![node],
                    ConfigFileNodeOrNodes::Nodes(nodes) => nodes,
                }
            } else {
                vec![]
            };

            let sort_rules = config_file
                .sort_rule
                .map(|rule_or_rules| match rule_or_rules {
                    ConfigFileSortRuleOrSortRules::Rule(rule) => SortRules::from(vec![rule]),
                    ConfigFileSortRuleOrSortRules::Rules(rules) => SortRules::from(rules),
                })
                .unwrap_or_else(SortRules::empty);

            let config_file_templates =
                if let Some(config_file_templates_from_config_file) = config_file.template {
                    match config_file_templates_from_config_file {
                        ConfigFileTemplateOrTemplates::Template(template) => {
                            config_file_templates_from_cli.push(template);
                            config_file_templates_from_cli
                        }
                        ConfigFileTemplateOrTemplates::Templates(templates) => {
                            config_file_templates_from_cli.extend(templates);
                            config_file_templates_from_cli
                        }
                    }
                } else {
                    config_file_templates_from_cli
                };

            let output_directory =
                if let Some(output_directory_from_cli) = cli_config.output_directory {
                    output_directory_from_cli
                } else {
                    PathBuf::from(".")
                };

            (
                providers,
                standalone_nodes,
                sort_rules,
                config_file_templates,
                output_directory,
            )
        } else {
            let output_directory =
                if let Some(output_directory_from_cli) = cli_config.output_directory {
                    output_directory_from_cli
                } else {
                    PathBuf::from(".")
                };
            (
                providers_from_cli,
                vec![],
                SortRules::empty(),
                config_file_templates_from_cli,
                output_directory,
            )
        };

    debug!(
        "templates from CLI arguments and config file:\n{:?}",
        &config_file_templates
    );

    let templates = {
        let template_futures = config_file_templates
            .into_iter()
            .map(|cft| async { cft.into_tempalte().await });

        let mut templates = try_join_all(template_futures)
            .await
            .context("failed to fetch templates")?;

        if matches!(cli_config.use_built_in_templates, Some(true))
            || (cli_config.use_built_in_templates.is_none() && templates.is_empty())
        {
            templates.extend(get_built_in_templates());
        }

        templates
    };

    let log_level = match cli_config.verbose {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    Ok(MergedConfig {
        providers,
        standalone_nodes,
        sort_rules,
        templates,
        output_directory,
        log_level,
    })
}

fn get_provider_from_cli_input(provider_type: &CliProviderType, url: Uri) -> Providers {
    match provider_type {
        CliProviderType::Ssr => Providers::Ssr(Ssr {
            name: None,
            url,
            options: Default::default(),
        }),
        CliProviderType::Clash => Providers::Clash(Clash {
            name: None,
            url,
            options: Default::default(),
        }),
    }
}
