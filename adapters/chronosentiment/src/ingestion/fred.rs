use std::error::Error;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value;

pub struct FredWorker {
    client: reqwest::Client,
    api_key: String,
}

impl FredWorker {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    /// Fetches a FRED series (e.g. 'CPIAUCSL' for CPI or 'UNRATE' for Unemployment)
    pub async fn fetch_series(&self, series_id: &str) -> Result<Vec<Value>, Box<dyn Error>> {
        let url = format!(
            "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json",
            series_id, self.api_key
        );

        let response = self.client.get(&url).send().await?.json::<Value>().await?;

        let observations_array = response["observations"].as_array().ok_or("Missing observations array")?;
        
        let mut observations = Vec::new();

        for obs in observations_array {
            let date_str = obs["date"].as_str().unwrap_or("");
            let value_str = obs["value"].as_str().unwrap_or("");

            if value_str == "." {
                continue; // FRED sometimes returns "." for missing data
            }

            if let (Ok(date), Ok(val)) = (date_str.parse::<NaiveDate>(), value_str.parse::<f64>()) {
                let observed_at = date.and_hms_opt(0, 0, 0).unwrap().and_utc();

                let observation = serde_json::json!({
                    "observation_type": "MacroRelease",
                    "observed_at": observed_at.to_rfc3339(),
                    "symbol": series_id,
                    "numerical_value": val,
                    "payload": {
                        "indicator": series_id,
                        "value": val
                    },
                    "source_name": "FRED",
                    "coverage": "Complete"
                });

                observations.push(observation);
            }
        }

        Ok(observations)
    }
}
