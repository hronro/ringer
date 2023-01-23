#![warn(clippy::all)]

use anyhow::{anyhow, Error, Result};
use futures::future::try_join_all;
use log::{debug, info, trace, warn};
use once_cell::sync::OnceCell;
use simple_logger::init_with_level as init_logger_with_level;

use cli::get_config;
use config::MergedConfig;
use node::Node;
use provider::Provider;
use template::{RenderEngine, TemplateArgs};

mod cli;
mod config;
mod node;
mod provider;
mod template;
mod utils;

static CONFIG: OnceCell<MergedConfig> = OnceCell::new();
static NODES_BY_PROVIDERS: OnceCell<Vec<Vec<Node>>> = OnceCell::new();
static TEMPLATE_ARGS: OnceCell<TemplateArgs> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<()> {
    CONFIG
        .set(get_config().await?)
        .map_err(|_| anyhow!("can't set CONFIG!"))?;
    let config = CONFIG.get().unwrap();

    init_logger_with_level(config.log_level)?;

    debug!("Config:\n{:#?}", &config);

    if config.providers.is_empty() {
        warn!("no providers");
    }

    let nodes_futures = config.providers.iter().map(|provider| async {
        debug!(
            "start fetching content of provider `{}`...",
            provider.get_display_name(),
        );
        let content = provider.fetch_content().await?;
        trace!(
            "content of provider `{}`:\n{:?}",
            provider.get_display_name(),
            &content
        );

        let nodes = provider.parse_nodes_from_content(content)?;

        debug!(
            "getting nodes of provider `{}`\n{:?}",
            provider.get_display_name(),
            &nodes
        );

        std::result::Result::<Vec<Node>, Error>::Ok(nodes)
    });

    NODES_BY_PROVIDERS
        .set(try_join_all(nodes_futures).await?)
        .map_err(|_| anyhow!("can't set NODES_BY_PROVIDERS!"))?;
    let nodes_by_providers = NODES_BY_PROVIDERS.get().unwrap();
    if !config.providers.is_empty() {
        info!("fetch providers complete");
    }

    TEMPLATE_ARGS
        .set(TemplateArgs::new(
            &config.providers,
            nodes_by_providers,
            &config.standalone_nodes,
        ))
        .map_err(|_| anyhow!("can't set TEMPLATE_ARGS!"))?;

    let template_args = TEMPLATE_ARGS.get().unwrap();

    debug!("template args:\n{:#?}", &TEMPLATE_ARGS);

    let mut render_engine = RenderEngine::new(template_args, &config.templates);
    render_engine.render(&config.output_directory)?;

    eprintln!("✅ Done!");
    Ok(())
}
