//! Thermostat controller -- fetch states, set mode and temperature.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::thermostat::{Thermostat, ThermostatMode};

/// Fetch all thermostats.
pub async fn fetch_thermostats(client: &mut AlarmClient) -> Result<Vec<Thermostat>> {
    let resp = client.get(ResourceType::Thermostat, None).await?;
    resp.resources()
        .into_iter()
        .map(Thermostat::from_resource)
        .collect()
}

/// Fetch a single thermostat by ID.
pub async fn fetch_thermostat(client: &mut AlarmClient, id: &str) -> Result<Thermostat> {
    let resp = client.get(ResourceType::Thermostat, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("thermostat {id} not found")))?;
    Thermostat::from_resource(resource)
}

/// Set the thermostat mode.
pub async fn set_mode(client: &mut AlarmClient, id: &str, mode: ThermostatMode) -> Result<()> {
    let state_code = match mode {
        ThermostatMode::Off => 1,
        ThermostatMode::Heat => 2,
        ThermostatMode::Cool => 3,
        ThermostatMode::Auto => 4,
        ThermostatMode::AuxHeat => 5,
        ThermostatMode::Unknown => {
            return Err(AlarmError::UnsupportedOperation(
                "cannot set thermostat to unknown mode".to_string(),
            ))
        }
    };

    let body = serde_json::json!({
        "statePollOnly": false,
        "state": state_code,
    });
    client
        .post(ResourceType::Thermostat, id, "setState", body)
        .await?;
    Ok(())
}

/// Set the heat setpoint.
pub async fn set_heat_setpoint(client: &mut AlarmClient, id: &str, temp: f64) -> Result<()> {
    let body = serde_json::json!({
        "statePollOnly": false,
        "heatSetpoint": temp,
    });
    client
        .post(ResourceType::Thermostat, id, "setHeatSetpoint", body)
        .await?;
    Ok(())
}

/// Set the cool setpoint.
pub async fn set_cool_setpoint(client: &mut AlarmClient, id: &str, temp: f64) -> Result<()> {
    let body = serde_json::json!({
        "statePollOnly": false,
        "coolSetpoint": temp,
    });
    client
        .post(ResourceType::Thermostat, id, "setCoolSetpoint", body)
        .await?;
    Ok(())
}
