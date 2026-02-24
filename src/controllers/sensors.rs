//! Sensor controller -- fetch sensor states.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::sensor::Sensor;

/// Fetch all sensors.
pub async fn fetch_sensors(client: &mut AlarmClient) -> Result<Vec<Sensor>> {
    let resp = client.get(ResourceType::Sensor, None).await?;
    resp.resources()
        .into_iter()
        .map(Sensor::from_resource)
        .collect()
}

/// Fetch a single sensor by ID.
pub async fn fetch_sensor(client: &mut AlarmClient, id: &str) -> Result<Sensor> {
    let resp = client.get(ResourceType::Sensor, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("sensor {id} not found")))?;
    Sensor::from_resource(resource)
}
