//! Garage door controller -- fetch states, open/close.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::garage_door::{GarageDoor, GarageDoorCommand};

/// Fetch all garage doors.
pub async fn fetch_garage_doors(client: &mut AlarmClient) -> Result<Vec<GarageDoor>> {
    let resp = client.get(ResourceType::GarageDoor, None).await?;
    resp.resources()
        .into_iter()
        .map(GarageDoor::from_resource)
        .collect()
}

/// Fetch a single garage door by ID.
pub async fn fetch_garage_door(client: &mut AlarmClient, id: &str) -> Result<GarageDoor> {
    let resp = client.get(ResourceType::GarageDoor, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("garage door {id} not found")))?;
    GarageDoor::from_resource(resource)
}

/// Open a garage door.
pub async fn open(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(
            ResourceType::GarageDoor,
            id,
            GarageDoorCommand::Open.api_action(),
            body,
        )
        .await?;
    Ok(())
}

/// Close a garage door.
pub async fn close(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(
            ResourceType::GarageDoor,
            id,
            GarageDoorCommand::Close.api_action(),
            body,
        )
        .await?;
    Ok(())
}
