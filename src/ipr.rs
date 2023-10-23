use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IprError(String),
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub ipr_id: String,
    pub parent: Option<String>,
    pub status: String,
}

fn get_ipr_data(
    ipr_url: &str,
    ipr_key: &str,
    lm_id: &str,
) -> Result<Data, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let json = client
        .get(format!("{ipr_url}/v2/products/{lm_id}"))
        .header("X-API-KEY", ipr_key)
        .send()?
        .text()?;

    match serde_json::from_str::<Data>(json.as_str()) {
        Ok(data) => Ok(data),
        Err(e) => Err(Box::new(e)),
    }
}

/// # Errors
///
/// Will return `Err` if gateway data cannot be fetched.
pub fn get_gw_data(
    ipr_url: &str,
    ipr_key: &str,
    lm_id: &str,
) -> Result<Data, Box<dyn std::error::Error>> {
    match get_ipr_data(ipr_url, ipr_key, lm_id) {
        Ok(lm_data) => {
            if let Some(gw_id) = lm_data.parent {
                get_ipr_data(ipr_url, ipr_key, gw_id.as_str())
            } else {
                Err(Box::new(Error::IprError(format!(
                    "Failed to get parent of {lm_id}"
                ))))
            }
        },
        Err(e) => Err(Box::new(Error::IprError(format!(
            "Failed to fetch IPR data for {lm_id}\nError: {e}"
        )))),
    }
}
