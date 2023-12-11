use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    CpmsError(String),
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub ipr_id: String,
    pub parent: Option<String>,
    pub status: String,
}

fn get_cpms_data(
    cpms_url: &str,
    cpms_user: &str,
    cpms_password: &str,
    ipr_id: &str,
) -> Result<Data, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let json = client
        .get(format!("{cpms_url}/api/iprconfig/{ipr_id}"))
        .basic_auth(cpms_user, Some(cpms_password))
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
    cpms_url: &str,
    cpms_user: &str,
    cpms_password: &str,
    lm_id: &str,
) -> Result<Data, Box<dyn std::error::Error>> {
    match get_cpms_data(cpms_url, cpms_user, cpms_password, lm_id) {
        Ok(lm_data) => {
            if let Some(gw_id) = lm_data.parent {
                get_cpms_data(cpms_url, cpms_user, cpms_password, gw_id.as_str())
            } else {
                Err(Box::new(Error::CpmsError(format!(
                    "Failed to get parent of {lm_id}"
                ))))
            }
        }
        Err(e) => Err(Box::new(Error::CpmsError(format!(
            "Failed to fetch CPMS data for {lm_id}\nError: {e}"
        )))),
    }
}
