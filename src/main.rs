use crate::discord::{say, start_bot};
use anyhow::Result;
use db::Db;
use dotenv::dotenv;
use env_logger::{Builder, Target};
use log::{error, info};
use poise::serenity_prelude as serenity;
use serenity::model::prelude::*;
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

mod db;
mod discord;
mod meteo;
mod powerfox;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    info!("Starting app.");
    dotenv().ok();
    // TODO the bot-permission-stiff is the important part

    // setup
    let db = Db::new().await?;
    let cloned_db = db.clone();
    let token = env::var("DISCORD_TOKEN")?;
    let intents = serenity::GatewayIntents::non_privileged();

    // schedule the daily message
    let sched = JobScheduler::new().await?;

    // schedule daily job
    sched
        .add(Job::new_async("0 12 * * * *", move |_uuid, mut _l| {
            let db = db.clone();
            Box::pin(async move {
                let token = env::var("DISCORD_TOKEN").expect("Could not find DISCORD_TOKEN");
                powerfox_daily(&token.clone(), &db)
                    .await
                    .expect("Could not execute daily task.");
            })
        })?)
        .await?;
    sched.start().await?;

    // start the bot
    // TODO check if we can use &Db
    start_bot(&token, intents, cloned_db).await?;

    // TODO include general costs
    // TODO what now?
    // -> analyze data (use history from git-repo or whatever): send warnings depending on consumption, price and temperature
    // => recognize trends, f.e. if consumption starts getting higher; temperature is higher than $THRESHOLD, but there still was significant consumption (f.e. more than 20kWh)
    // -> make variables configurable by Discord-commands or mail -> thresholds, costs, etc.
    // do some of the above daily (cost compared to yesterday, a week before) and monthly (broader trends, etc.)
    // TODO host this publicly on gitlab or github to get dependabot-PRs
    Ok(())
}

async fn powerfox_daily(token: &str, db: &Db) -> Result<()> {
    info!("Writing yesterday's data to Discord.");
    let channel_id = ChannelId::new(env::var("DISCORD_CHANNEL_ID")?.parse()?);
    say(token, channel_id, "Getting yesterday's data.".to_string()).await?;

    match db.create_yesterday().await {
        Ok(day) => {
            let config = db.get_config().await?;
            say(token, channel_id, day.summary(&config.cost_heating)).await?;

            let days = db.get_days_of_month().await?;
            say(token, channel_id, days.summary(&config)?).await?;
            info!("Done with daily data and summary.")
        }
        Err(err) => {
            let error = format!("Encountered an error: {}", err);
            error!("{}", error);
            say(token, channel_id, error).await?;
        }
    }
    Ok(())
}
