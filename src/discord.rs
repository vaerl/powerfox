use anyhow::Result;
use chrono::format;
use poise::serenity_prelude as serenity;
use serenity::{model::prelude::*, Client};
use std::env;

use crate::db::Db;

pub struct Discord {
    client: Client,
    channel_id: ChannelId,
}

impl Discord {
    /// Creates the serenity-Discord-client.
    /// Db is a clone of the previously created Db, more on cloning a pool here: https://github.com/launchbadge/sqlx/discussions/917
    pub async fn new(db: Db) -> Result<Self> {
        let token = env::var("DISCORD_TOKEN")?;
        let channel_id = ChannelId::new(env::var("DISCORD_CHANNEL_ID")?.parse()?);
        let intents =
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                // NOTE this error seems okay - it doesn't show up when running cargo check or the like
                commands: vec![age(), app_info()],
                prefix_options: poise::PrefixFrameworkOptions {
                    prefix: Some("!".into()),
                    additional_prefixes: vec![
                        poise::Prefix::Literal("hey bot"),
                        poise::Prefix::Literal("hey bot,"),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data { db })
                })
            })
            .build();

        let mut client = serenity::ClientBuilder::new(token, intents)
            .framework(framework)
            .await?;
        client.start().await?;
        Ok(Discord { client, channel_id })
    }

    pub async fn say(&self, content: String) -> Result<()> {
        self.channel_id
            .say(self.client.http.clone(), content)
            .await?;

        Ok(())
    }
}

struct Data {
    db: Db,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn app_info(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "Running version {}.",
        env::var("CARGO_PKG_VERSION").unwrap_or("<could not find version>".to_string())
    ))
    .await?;
    Ok(())
}
