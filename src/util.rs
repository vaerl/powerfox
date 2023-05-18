use chrono::NaiveDateTime;
use serde::Deserialize;

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<Vec<NaiveDateTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Vec<String> = Deserialize::deserialize(deserializer)?;
    let dt: Result<Vec<NaiveDateTime>, _> = s
        .iter()
        .map(|s| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
        })
        .collect();
    dt
}
