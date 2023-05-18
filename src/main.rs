use anyhow::Result;
use dotenv::dotenv;
use meteo::Meteo;

use crate::powerfox::Powerfox;

mod db;
mod meteo;
mod powerfox;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let powerfox = Powerfox::new()?;
    let meteo = Meteo::new()?;
    let devices = powerfox.get_devices().await?;

    for device in devices {
        let report = powerfox.get_report_for(&device.device_id).await?;
        println!(
            "Consumption for device '{}': {:#?}",
            device.name, report.consumption.sum
        );
    }

    let temperature = meteo.get_temperature_for_today().await?;

    // TODO what then?
    // -> configure costs for both heating- and general power -> use a db and run with docker-compose
    // => sqlx,
    // -> calculate costs for the given consumption
    // -> commit this to some document or something, possibly just push to some git-repo
    // -> send a message to somewhere, possibly Discord: https://github.com/serenity-rs/serenity
    // => heating in kWh and cost, general in kWh and cost, (remaining) monthly budget
    // -> analyze data (use history from git-repo or whatever): send warnings depending on consumption, price and temperature
    // => recognize trends, f.e. if consumption starts getting higher; temperature is higher than $THRESHOLD, but there still was significant consumption (f.e. more than 20kWh)
    // -> make variables configurable by Discord-commands or mail -> thresholds, costs, etc.
    // TODO host this publicly on gitlab or github to get dependabot-PRs
    Ok(())
}
