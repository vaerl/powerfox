use crate::{meteo::Meteo, powerfox::{Powerfox, Report}};
use anyhow::{anyhow, bail, Result};
use chrono::{Datelike, Local, NaiveDate, Duration};
use log::info;
use serenity::utils::MessageBuilder;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    types::Uuid,
    PgPool,
};
use std::env;

#[derive(sqlx::FromRow)]
pub struct Config {
    pub id: Uuid,
    pub cost_heating: f64,
    pub cost_general: f64,
    pub monthly_budget_heating: f64,
    pub monthly_budget_general: f64,
}

impl Config {

    /// Create a new [Config] with an updated value for [cost_heating].
    pub fn with_cost_heating(self, cost_heating: f64) -> Self {
        Config { id: self.id, cost_heating, cost_general: self.cost_general, monthly_budget_heating: self.monthly_budget_heating, monthly_budget_general: self.monthly_budget_general }
    }

    /// Create a new [Config] with an updated value for [cost_general].
    pub fn with_cost_general(self, cost_general: f64) -> Self {
        Config { id: self.id, cost_heating: self.cost_heating, cost_general, monthly_budget_heating: self.monthly_budget_heating, monthly_budget_general: self.monthly_budget_general }
    }

    /// Create a new [Config] with an updated value for [monthly_budget_heating].
    pub fn with_monthly_budget_heating(self, monthly_budget_heating: f64) -> Self {
        Config { id: self.id, cost_heating: self.cost_heating, cost_general: self.cost_general, monthly_budget_heating, monthly_budget_general: self.monthly_budget_general }
    }
    /// Create a new [Config] with an updated value for [monthly_budget_general].
    pub fn with_monthly_budget_general(self, monthly_budget_general: f64) -> Self {
        Config { id: self.id, cost_heating: self.cost_heating, cost_general: self.cost_general, monthly_budget_heating: self.monthly_budget_heating, monthly_budget_general }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Day {
    id: Uuid,
    // TODO maybe have time-series here at some point and calculate average_temperature and such?
    pub heating_consumption: f64,
    pub general_consumption: f64,
    pub average_temperature: f64,
    pub date: NaiveDate,
}

pub struct CreateDay {
    pub heating_consumption: f64,
    pub general_consumption: f64,
    pub average_temperature: f64,
    pub date: NaiveDate,
}

pub struct Days(Vec<Day>);

impl Day {
    pub fn new(heating_report: Report, general_report: Report, average_temperature: f64) -> Self {
        Day {
            id: Uuid::new_v4(),
            heating_consumption: heating_report.consumption.sum,
            general_consumption: general_report.consumption.sum,
            average_temperature,
            date: Local::now().date_naive(),
        }
    }

    pub fn summary(&self, cost_heating: &f64) -> String {
        MessageBuilder::new()
                .push_line_safe("Done getting data - here's your daily summary:")
                .push_quote("With a temperature of ")
                .push_bold_safe(format!("{:.2} °C", self.average_temperature))
                .push(", you've used ")
                .push_bold_safe(format!("{:.2} kWh", self.heating_consumption))
                .push(" for heating - this cost ")
                .push_bold_safe(format!("{:.2} €", self.heating_cost(cost_heating)))
                .push(".")
                .build()
    }

    pub fn heating_cost(&self, cost: &f64) -> f64 {
        self.heating_consumption * cost
    }

    pub fn general_cost(&self, cost: &f64) -> f64 {
        self.general_consumption * cost
    }
}

impl Days {
    pub fn summary(&self, config: &Config) -> Result<String> {
        match self.heating_cost(&config.cost_heating) {
            Ok(total_heating_cost) => {
                let message = MessageBuilder::new()
                    .push_quote("This month, you've used ")
                    .push_bold_safe(format!("{:.2} €", total_heating_cost))
                    .push(" of ")
                    .push_bold_safe(format!("{:.2} €", config.monthly_budget_heating))
                    .push(".")
                    .build();
                Ok(message)
            }
            _ =>  bail!("Could not reduce costs to calculate total cost."),
        }
    }

    pub fn heating_cost(&self, cost: &f64) -> Result<f64> {
        let consumption = self.0.iter().map(|d| d.heating_cost(cost)).reduce(|a, b| a + b);
        match consumption {
            Some(val) => Ok(val),
            None => bail!("Could not calculate heating-consumption."),
        }
    }

    pub fn general_cost(&self, cost: &f64) -> Result<f64> {
        let consumption = self.0.iter().map(|d| d.general_cost(cost)).reduce(|a, b| a + b);
        match consumption {
            Some(val) => Ok(val),
            None => bail!("Could not calculate general-consumption."),
        }
    }
}

impl CreateDay {
    pub fn yesterday(heating_report: Report, general_report: Report, average_temperature: f64) -> Self {
        CreateDay {
            heating_consumption: heating_report.consumption.sum,
            general_consumption: general_report.consumption.sum,
            average_temperature,
            date: Local::now().date_naive() - Duration::days(1),
        }
    }
}

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}


impl Db {
    pub async fn new() -> Result<Self> {
        let username = env::var("DATABASE_USER")?;
        let password = env::var("DATABASE_PASSWORD")?;
        let host = env::var("DATABASE_HOST")?;
        let port: u16 = env::var("DATABASE_PORT")?.parse()?;
        let database = env::var("DATABASE_TABLE")?;

        let options = PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&username)
            .password(&password)
            .database(&database);

        let pool = PgPoolOptions::new()
            .max_connections(5) 
            .connect_with(options)
            .await?;

        info!("Set up database-client.");
        Ok(Db {pool})
    }

    /// Updates the event if it exists, otherwise just adds it to the database.
    pub async fn save_yesterday(&self, day: CreateDay) -> Result<Day> {
        match self.get_yesterday().await {
            Ok(existing_day) => {
                // we already have a day, no need to create another
                self.update_day(existing_day.id, day).await
            },
            Err(_) => {
                let day = sqlx::query_as!(Day, 
                    "INSERT INTO days (id, heating_consumption, general_consumption, average_temperature, date) VALUES ($1, $2, $3, $4, $5) RETURNING id, heating_consumption, general_consumption, average_temperature, date",
                    Uuid::new_v4(), day.heating_consumption, day.general_consumption, day.average_temperature, day.date).fetch_one(&self.pool).await?;
                Ok(day)
            },
        }
    }

    pub async fn update_day(&self, id: Uuid, day: CreateDay) -> Result<Day> {
        let day = sqlx::query_as!(Day, 
            "UPDATE days SET (heating_consumption, general_consumption, average_temperature, date) = ($1, $2, $3, $4) WHERE id = $5 RETURNING id, heating_consumption, general_consumption, average_temperature, date",
            day.heating_consumption, day.general_consumption, day.average_temperature, day.date, id).fetch_one(&self.pool).await?;
        Ok(day)
    }

    /// Get a specific day from the database.
    pub async fn get_day(&self, date: NaiveDate) -> Result<Day> {
        let day = sqlx::query_as!(Day, "SELECT * FROM days WHERE date = $1", date)
            .fetch_one(&self.pool)
            .await?;
        Ok(day)
    }

    /// Wrapper around [get_day](Db::get_day).
    pub async fn get_yesterday(&self) -> Result<Day> {
        let yesterday = Local::now().date_naive() - Duration::days(1);
        self.get_day(yesterday).await
    }

    /// Create the entry for yesterday. Loads from the database if it exists already
    pub async fn create_yesterday(&self) -> Result<Day> {
            // if we have the data already, just return it to save on API-calls
    if let Ok(day) = self.get_yesterday().await {
        return Ok(day);
    }

    let meteo = Meteo::new()?;
    let temperature = meteo.get_temperature_for_yesterday().await?;

    let powerfox = Powerfox::new()?;
    let devices = powerfox.get_devices().await?;

    let mut heating_report = None;
    let mut general_report = None;
    for device in devices {
        if device.name == "Heizstrom" {
            heating_report = Some(powerfox.get_report_for_yesterday(&device.device_id).await?);
        }

        if device.name == "Allgemeinstrom" {
            general_report = Some(powerfox.get_report_for_yesterday(&device.device_id).await?);
        }

        // checking this in the loop might be beneficial if there are multiple other devices
        if heating_report.is_some() && general_report.is_some() {
            let yesterday = CreateDay::yesterday(
                // we can just unwrap here because we check with is_some() before
                heating_report.unwrap(),
                general_report.unwrap(),
                temperature.average_temperature()?,
            );

            return self.save_yesterday(yesterday).await;
        }
    }

    Err(anyhow!(
        "Could not get all necessary data for the current day."
    ))
    }

    pub async fn get_days_of_month(&self) -> Result<Days> {
        let current_date = Local::now().naive_local();
        let first_of_month = NaiveDate::from_ymd_opt(current_date.year(), current_date.month(), 1)
            .ok_or(anyhow!("Could not create date for this month."))?;

        let days = sqlx::query_as!(Day, "SELECT * FROM days WHERE date > $1", first_of_month)
            .fetch_all(&self.pool)
            .await?;

        Ok(Days(days))
    }

    /// Get the current config from the database.
    pub async fn get_config(&self) -> Result<Config> {
        let config = sqlx::query_as!(Config, "SELECT * FROM config")
            .fetch_one(&self.pool)
            .await?;
        Ok(config)
    }

    /// Update the existing config.
    pub async fn update_config(&self, id: Uuid, config: Config) -> Result<Config> {
        let config = sqlx::query_as!(Config, 
            "UPDATE config SET (cost_heating, cost_general, monthly_budget_heating, monthly_budget_general) = ($1, $2, $3, $4) WHERE id = $5 RETURNING id, cost_heating, cost_general, monthly_budget_heating, monthly_budget_general", config.cost_heating, config.cost_general, config.monthly_budget_heating, config.monthly_budget_general, id)
            .fetch_one(&self.pool)
            .await?;
        Ok(config)
    }
}
