use crate::powerfox::Powerfox;
use anyhow::{anyhow, Result};
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use db::{CreateDay, Day, Db};
use discord::Discord;
use dotenv::dotenv;
use meteo::Meteo;
use reqwest::StatusCode;
use std::{env, net::SocketAddr};

mod db;
mod discord;
mod meteo;
mod powerfox;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let port: u16 = env::var("APP_PORT")?.parse()?;

    let app = Router::new().route("/powerfox", get(get_wrapper));
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    // TODO trigger with systemd
    // TODO what now?
    // -> analyze data (use history from git-repo or whatever): send warnings depending on consumption, price and temperature
    // => recognize trends, f.e. if consumption starts getting higher; temperature is higher than $THRESHOLD, but there still was significant consumption (f.e. more than 20kWh)
    // -> make variables configurable by Discord-commands or mail -> thresholds, costs, etc.
    // TODO host this publicly on gitlab or github to get dependabot-PRs
    Ok(())
}

async fn get_wrapper() -> Result<(), AppError> {
    let discord = Discord::new().await?;
    let db = Db::new().await?;

    discord.say(format!("Getting yesterday's data.")).await?;
    match get_and_write_data(&db).await {
        Ok(day) => {
            let config = db.get_config().await?;
            discord.say(day.summary(&config.cost_heating)).await?;

            let days = db.get_days_of_month().await?;
            discord.say(days.summary(&config)?).await?
        }
        Err(err) => {
            discord
                .say(format!("Encountered an error: {}", err))
                .await?
        }
    }
    Ok(())
}

async fn get_and_write_data(db: &Db) -> Result<Day> {
    // if we have the data already, just return it to save on API-calls
    if let Ok(day) = db.get_yesterday().await {
        return Ok(day);
    }

    let meteo = Meteo::new()?;
    let temperature = meteo.get_temperature_for_yesterday().await?;

    let powerfox = Powerfox::new()?;
    let devices = powerfox.get_devices().await?;

    if devices.len() != 2 {
        return Err(anyhow!("Found more than two devices, aborting."));
    }

    let mut heating_report = None;
    let mut general_report = None;
    for device in devices {
        if device.name == "Heizstrom" {
            heating_report = Some(powerfox.get_report_for_yesterday(&device.device_id).await?);
        }

        if device.name == "Allgemeinstrom" {
            general_report = Some(powerfox.get_report_for_yesterday(&device.device_id).await?);
        }

        if heating_report.is_some() && general_report.is_some() {
            let yesterday = CreateDay::yesterday(
                // we can just unwrap here because we check with is_some() before
                heating_report.unwrap(),
                general_report.unwrap(),
                temperature.average_temperature()?,
            );

            return db.save_yesterday(yesterday).await;
        }
    }
    Err(anyhow!(
        "Could not get all necessary data for the current day."
    ))
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
