use anyhow::Result;
use serenity::{model::prelude::*, Client};
use std::env;

pub struct Discord {
    client: Client,
    channel_id: ChannelId,
}

impl Discord {
    /// Creates the serenity-Discord-client.
    pub async fn new() -> Result<Self> {
        let token = env::var("DISCORD_TOKEN")?;
        let channel_id = ChannelId(env::var("DISCORD_CHANNEL_ID")?.parse()?);
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(&token, intents).await?;
        Ok(Discord { client, channel_id })
    }

    pub async fn say(&self, content: String) -> Result<()> {
        self.channel_id
            .say(self.client.cache_and_http.clone().http.clone(), content)
            .await?;

        Ok(())
    }
}
