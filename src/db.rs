use crate::powerfox::Report;
use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    types::Uuid,
    PgPool,
};
use std::env;

#[derive(sqlx::FromRow)]
struct Config {
    id: Uuid,
    pub cost_heating: f64,
    pub cost_general: f64,
    pub monthly_budget_heating: f64,
    pub monthly_budget_general: f64,
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
}

struct Db {
    username: String,
    password: String,
    host: String,
    port: u16,
    pool: PgPool,
}

impl Db {
    async fn new() -> Result<Self> {
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
            .max_connections(5) // Adjust the maximum number of connections as needed
            .connect_with(options)
            .await?;

        Ok(Db {
            username,
            password,
            host,
            port,
            pool,
        })
    }

    /// Get a specific day from the database.
    pub async fn get_day(&self, date: NaiveDate) -> Result<Day> {
        let day = sqlx::query_as!(Day, "SELECT * FROM days WHERE date = $1", date)
            .fetch_one(&self.pool)
            .await?;
        Ok(day)
    }

    /// Wrapper around [get_day](Db::get_day).
    pub async fn get_today(&self) -> Result<Day> {
        self.get_day(Local::now().date_naive()).await
    }

    /// Get all the days from the database.
    pub async fn get_days(&self) -> Result<Vec<Day>> {
        let days = sqlx::query_as!(Day, "SELECT * FROM days")
            .fetch_all(&self.pool)
            .await?;
        Ok(days)
    }

    pub async fn get_days_of_month(&self) -> Result<Vec<Day>> {
        let current_date = Local::now().naive_local();
        let first_of_month = NaiveDate::from_ymd_opt(current_date.year(), current_date.month(), 1)
            .ok_or(anyhow!("Could not create date for this month."))?;

        let days = sqlx::query_as!(Day, "SELECT * FROM days WHERE date > $1", first_of_month)
            .fetch_all(&self.pool)
            .await?;
        Ok(days)
    }
}
