mod handler;
mod roles;

use serenity::all::{ChannelId, GatewayIntents};

use crate::handler::Handler;
use crate::roles::send_role_panel;

const TOKEN_ENV: &str = "MGDS_ROLE_CHOOSER_TOKEN";

#[tokio::main]
async fn main() -> serenity::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("mgds role chooser");

    let token = get_token();

    let intents = GatewayIntents::GUILDS;

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    if let Ok(channel_id) = std::env::var("ROLE_PANEL_CHANNEL_ID") {
        let channel_id: u64 = channel_id
            .parse()
            .expect("ROLE_PANEL_CHANNEL_ID must be a valid channel ID");

        send_role_panel(&client.http, ChannelId::new(channel_id))
            .await
            .unwrap_or_else(|error| tracing::error!("Failed to send role panel: {error}"));
    }

    client.start().await
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
