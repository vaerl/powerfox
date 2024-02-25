use ::serenity::all::Http;
use anyhow::Result;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::*;
use std::env;

use crate::{
    db::{Day, Db},
    meteo::Meteo,
    powerfox::Powerfox,
};

pub async fn start_bot(token: &str, intents: GatewayIntents, db: Db) -> Result<()> {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![version(), yesterday(), today(), budget(), costs()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
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
    Ok(())
}

pub async fn say(token: &str, channel_id: ChannelId, content: impl Into<String>) -> Result<()> {
    let cache_http = Http::new(token);
    channel_id.say(cache_http, content).await?;
    Ok(())
}

pub struct Data {
    db: Db,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Display the bot's version.
#[poise::command(slash_command, prefix_command)]
async fn version(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "The bot-version is {}.",
        env::var("CARGO_PKG_VERSION").unwrap_or("<could not find version>".to_string())
    ))
    .await?;
    Ok(())
}

/// Display yesterday's heating-info.
#[poise::command(slash_command, prefix_command)]
async fn yesterday(ctx: Context<'_>) -> Result<(), Error> {
    let yesterday = ctx.data().db.create_yesterday().await?;
    let config = ctx.data().db.get_config().await?;
    ctx.say(format!("{}", yesterday.summary(&config.cost_heating)))
        .await?;
    Ok(())
}

/// Display today's heating-info.
#[poise::command(slash_command, prefix_command)]
async fn today(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data().db.get_config().await?;
    let meteo = Meteo::new()?;
    let temperature = meteo.get_temperature_for_today().await?;

    let powerfox = Powerfox::new()?;
    let devices = powerfox.get_devices().await?;

    let mut heating_report = None;
    let mut general_report = None;

    // TODO this works, but it might be nice to have this unified somewhere
    for device in devices {
        if device.name == "Heizstrom" {
            heating_report = Some(powerfox.get_report_for_today(&device.device_id).await?);
        }

        if device.name == "Allgemeinstrom" {
            general_report = Some(powerfox.get_report_for_today(&device.device_id).await?);
        }
    }

    if heating_report.is_some() && general_report.is_some() {
        // NOTE we can't save this as the day isn't over yet
        let day = Day::new(
            heating_report.unwrap(),
            general_report.unwrap(),
            temperature.average_temperature()?,
        );
        ctx.say(day.summary(&config.cost_heating)).await?;
    } else {
        ctx.say("Something went wrong.").await?;
    }

    Ok(())
}

/// Display the configured budgets.
#[poise::command(slash_command, prefix_command)]
async fn budget(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data().db.get_config().await?;
    ctx.say(format!(
        "Heating-Budget: {}€\nGeneral Budget: {}€",
        config.monthly_budget_heating, config.monthly_budget_general
    ))
    .await?;
    Ok(())
}

/// Display this month's costs.
#[poise::command(slash_command, prefix_command)]
async fn costs(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data().db.get_config().await?;
    let days = ctx.data().db.get_days_of_month().await?;
    ctx.say(format!(
        "Cost of Heating: {:.2}€/{}€\nGeneral cost: {:.2}€/{}€",
        days.heating_cost(&config.cost_heating)?,
        config.monthly_budget_heating,
        days.general_cost(&config.cost_general)?,
        config.monthly_budget_general
    ))
    .await?;
    Ok(())
}
