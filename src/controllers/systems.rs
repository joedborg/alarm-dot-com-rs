//! System controller -- fetches system info and discovers device IDs.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::system::System;

/// Fetch available system items to discover the active system ID.
pub async fn fetch_available_systems(client: &mut AlarmClient) -> Result<Vec<String>> {
    let resp = client.get_path("systems/availableSystemItems").await?;
    let resources = resp.resources();

    let ids: Vec<String> = resources.iter().map(|r| r.id.clone()).collect();

    if ids.is_empty() {
        return Err(AlarmError::AuthenticationFailed(
            "no systems found for this account".to_string(),
        ));
    }

    Ok(ids)
}

/// Fetch a specific system by ID.
pub async fn fetch_system(client: &mut AlarmClient, system_id: &str) -> Result<System> {
    let resp = client.get(ResourceType::System, Some(system_id)).await?;

    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnexpectedResponse("empty system response".to_string()))?;

    System::from_resource(resource)
}

/// Fetch the first available system (most common case: single-system accounts).
pub async fn fetch_primary_system(client: &mut AlarmClient) -> Result<System> {
    let system_ids = fetch_available_systems(client).await?;
    let primary_id = &system_ids[0];
    fetch_system(client, primary_id).await
}
