#[macro_use]
extern crate tracing;

use std::{collections::HashSet, env};

use anyhow::{Context, Result};
use serenity::{model::prelude::UserId, prelude::*};
use tracing_subscriber::util::SubscriberInitExt;

mod commands;
mod handler;
mod log;

// note: this value is mirrored in src/commands/help.rs
pub const EMBED_COLOR: [u8; 3] = [0x58, 0x65, 0xF2];

async fn client() -> Result<Client> {
    let token =
        env::var("DISCORD_TOKEN").context("failed to load `DISCORD_TOKEN` environment variable")?;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let owners_raw = env::var("BOT_OWNERS");
    let owners: HashSet<UserId> = if let Ok(owners_raw) = owners_raw {
        owners_raw
            .split(',')
            .map(|id_string| {
                UserId(id_string.parse().expect(
                    "`BOT_OWNERS` environment variable values should be valid Discord user IDs",
                ))
            })
            .collect()
    } else {
        HashSet::new()
    };

    let client = Client::builder(token, intents)
        .event_handler(handler::Handler)
        .framework(commands::framework(owners).await)
        .await
        .expect("Discord client should build successfully");

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    log::logger()
        .try_init()
        .expect("logger initialization shouldn't fail");

    let mut client = client().await.context("failed to build client")?;
    client.start().await.context("client error occurred")?;

    Ok(())
}
