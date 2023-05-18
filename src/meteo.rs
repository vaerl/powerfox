use crate::util::deserialize_datetime;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::env;
pub struct Meteo {
    client: Client,
    base_url: String,
    latitude: String,
    longitude: String,
}

impl Meteo {
    pub fn new() -> Result<Self> {
        Ok(Meteo {
            client: Client::new(),
            base_url: env::var("WEATHER_BASE_URL")?,
            // TODO check if parsing here makes sense, Strings might suffice
            latitude: env::var("WEATHER_LATITUDE")?,
            longitude: env::var("WEATHER_LONGITUDE")?,
        })
    }

    /// Gets the temperature for this day from 00:00 to 23:00.
    ///
    /// See [this URL](https://api.open-meteo.com/v1/forecast?latitude=51.28&longitude=8.87&hourly=temperature_2m&forecast_days=1) for more information.
    pub async fn get_temperature_for_today(&self) -> Result<TemperatureData> {
        let response = self
            .client
            .get(format!(
                "{}/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&forecast_days=1",
                &self.base_url, &self.latitude, &self.longitude
            ))
            .send()
            .await?;
        if response.status() != StatusCode::OK {
            Err(anyhow!(
                "Status-Code of response was not OK: {}",
                response.text().await?
            ))
        } else {
            Ok(response.json().await?)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemperatureData {
    latitude: f32,
    longitude: f32,
    generationtime_ms: f32,
    // offset could be negative, maybe
    utc_offset_seconds: i32,
    timezone: String,
    timezone_abbreviation: String,
    elevation: f32,
    hourly_units: HourlyUnit,
    hourly: Hourly,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HourlyUnit {
    time: String,
    temperature_2m: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hourly {
    #[serde(deserialize_with = "deserialize_datetime")]
    time: Vec<NaiveDateTime>,

    /// The temperature two meters above ground.
    temperature_2m: Vec<f32>,
}
