use anyhow::Result;
use poise::serenity_prelude as serenity;
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
        let channel_id = ChannelId::new(env::var("DISCORD_CHANNEL_ID")?.parse()?);
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                // NOTE this error seems okay - it doesn't show up when running cargo check or the like
                commands: vec![age()],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data {})
                })
            })
            .build();

        let client = serenity::ClientBuilder::new(token, intents)
            .framework(framework)
            .await?;

        //let client = Client::builder(&token, intents).await?;
        Ok(Discord { client, channel_id })
    }

    pub async fn say(&self, content: String) -> Result<()> {
        self.channel_id
            .say(self.client.http.clone(), content)
            .await?;

        Ok(())
    }
}

struct Data {} // User data, which is stored and accessible in all command invocations
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
