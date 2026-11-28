mod handler;
mod roles;

use std::error::Error;
use std::sync::Arc;

use serenity::all::GatewayIntents;

use crate::handler::Handler;
use crate::roles::{Config, send_role_panel};

const TOKEN_ENV: &str = "MGDS_ROLE_CHOOSER_TOKEN";
const CONFIG_PATH_ENV: &str = "MGDS_ROLE_CHOOSER_CONFIG_PATH";
const DEFAULT_CONFIG_PATH: &str = "config.yaml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("mgds role chooser");

    let token = get_token();
    let config = load_config();

    let intents = GatewayIntents::GUILDS;

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler::new(Arc::new(config.clone())))
        .await?;

    'panels: for panel in config.panels() {
        for channel_id in panel.channel_ids() {
            if let Err(error) = send_role_panel(&client.http, *channel_id, panel).await {
                tracing::error!(
                    panel_id = panel.id,
                    channel_id = %channel_id,
                    "Failed to send role panel: {error}"
                );
                break 'panels;
            }
            tracing::info!(panel_id = panel.id, channel_id = %channel_id, "Role panel sent");
        }
    }

    client.start().await?;

    Ok(())
}

fn get_token() -> String {
    match std::env::var(TOKEN_ENV) {
        Ok(token) if !token.trim().is_empty() => token,
        Ok(_) => {
            tracing::error!("{TOKEN_ENV} is set but empty");
            std::process::exit(1);
        }
        Err(error) => {
            tracing::error!("failed to read {TOKEN_ENV}: {error}");
            std::process::exit(1);
        }
    }
}

fn load_config() -> Config {
    let path = std::env::var(CONFIG_PATH_ENV).unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());

    match Config::load(&path) {
        Ok(config) => {
            tracing::info!(
                path = path,
                panels = config.panels().len(),
                "Loaded bot configuration"
            );
            config
        }
        Err(error) => {
            tracing::error!("Failed to load configuration: {error}");
            std::process::exit(1);
        }
    }
}
