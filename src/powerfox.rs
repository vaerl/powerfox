use std::env;

use anyhow::Result;
use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::{Datelike, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub struct Powerfox {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl Powerfox {
    /// Creates a new client to interact with the Powerfox-API.
    /// Automatically reads `POWERFOX_BASE_URL`, `POWERFOX_USERNAME` and `POWERFOX_PASSWORD` for a `.env`-file.
    ///
    /// API-Docs are available [here](https://www.powerfox.energy/wp-content/uploads/2020/05/powerfox-Kunden-API.pdf).
    pub fn new() -> Result<Self> {
        Ok(Powerfox {
            client: Client::new(),
            base_url: env::var("POWERFOX_BASE_URL")?,
            username: env::var("POWERFOX_USERNAME")?,
            password: env::var("POWERFOX_PASSWORD")?,
        })
    }

    /// Get all devices linked to the specified account.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let response = self
            .client
            .get(format!("{}/api/2.0/my/all/devices", &self.base_url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        // NOTE type-inference had some hiccups when using the method's return-type, so we just use a typ-annotated variable
        let devices: Vec<Device> = response.json().await?;
        Ok(devices)
    }

    /// Get the values of all devices for the last 24 hours.
    pub async fn get_report(&self) -> Result<Report> {
        let response = self
            .client
            .get(format!("{}/api/2.0/my/all/report", &self.base_url))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        let report: Report = response.json().await?;
        Ok(report)
    }

    /// Get the values of the specified device for the last 24 hours.
    pub async fn get_report_for(&self, device_id: &String) -> Result<Report> {
        let response = self
            .client
            .get(format!(
                "{}/api/2.0/my/{}/report",
                &self.base_url, device_id
            ))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        let report: Report = response.json().await?;
        Ok(report)
    }

    /// Get the values of the specified device for the specified day (00:00  to 23:59).
    pub async fn get_report_for_day(&self, device_id: &String, date: NaiveDate) -> Result<Report> {
        let response = self
            .client
            .get(format!(
                "{}/api/2.0/my/{}/report?year={}&month={}&day={}",
                &self.base_url,
                device_id,
                date.year(),
                date.month(),
                date.day()
            ))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        let report: Report = response.json().await?;
        Ok(report)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Device {
    pub device_id: String,
    pub name: String,

    // TODO add some comment about this and its import
    #[serde(deserialize_with = "from_ts")]
    pub account_associated_since: chrono::DateTime<Utc>,
    pub main_device: bool,
    pub prosumer: bool,
    pub division: Division,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(i8)]
pub enum Division {
    NoType = -1,
    ElectricityMeter = 0,
    ColdWaterMeter = 1,
    WarmWaterMeter = 2,
    WarmthMeter = 3,
    GasMeter = 4,
    ColdAndWarmWaterMeter = 5,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Report {
    pub consumption: ValueWrapper,
    pub feed_in: ValueWrapper,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ValueWrapper {
    #[serde(deserialize_with = "from_ts")]
    pub start_time: chrono::DateTime<Utc>,

    // this field houses values that are non-standard timestamps, so make sure to capture correctly
    // - even if this takes up more memory
    pub start_time_currency: i128,

    // both sum and max apparently are in kWh
    pub sum: f64,
    pub max: f64,

    pub max_currency: f64,
    pub meter_readings: Vec<String>,
    pub report_values: Vec<ReportValue>,
    pub sum_currency: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ReportValue {
    pub device_id: String,

    #[serde(deserialize_with = "from_ts")]
    pub timestamp: chrono::DateTime<Utc>,

    complete: bool,
    pub delta: f64,

    // NOTE both delta_ht and delta_nt seem to be only present for consumption of Heizstrom, so I've decided to just use one struct
    #[serde(rename = "DeltaHT")]
    pub delta_ht: Option<f64>,
    #[serde(rename = "DeltaNT")]
    pub delta_nt: Option<f64>,
    pub delta_currency: f64,
    pub values_type: usize,
}
