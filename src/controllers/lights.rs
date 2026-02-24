//! Light controller -- fetch states, turn on/off, set brightness.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::light::{Light, LightCommand};

/// Fetch all lights.
pub async fn fetch_lights(client: &mut AlarmClient) -> Result<Vec<Light>> {
    let resp = client.get(ResourceType::Light, None).await?;
    resp.resources()
        .into_iter()
        .map(Light::from_resource)
        .collect()
}

/// Fetch a single light by ID.
pub async fn fetch_light(client: &mut AlarmClient, id: &str) -> Result<Light> {
    let resp = client.get(ResourceType::Light, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("light {id} not found")))?;
    Light::from_resource(resource)
}

/// Turn a light on.
pub async fn turn_on(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(
            ResourceType::Light,
            id,
            LightCommand::TurnOn.api_action(),
            body,
        )
        .await?;
    Ok(())
}

/// Turn a light off.
pub async fn turn_off(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(
            ResourceType::Light,
            id,
            LightCommand::TurnOff.api_action(),
            body,
        )
        .await?;
    Ok(())
}

/// Set a dimmer light's brightness level (0-100).
pub async fn set_brightness(client: &mut AlarmClient, id: &str, level: u32) -> Result<()> {
    let body = serde_json::json!({
        "statePollOnly": false,
        "lightLevel": level.min(100),
    });
    client
        .post(ResourceType::Light, id, "setLevel", body)
        .await?;
    Ok(())
}
