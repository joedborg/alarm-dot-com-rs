//! Lock controller -- fetch lock states, lock/unlock.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::lock::{Lock, LockCommand};

/// Fetch all locks.
pub async fn fetch_locks(client: &mut AlarmClient) -> Result<Vec<Lock>> {
    let resp = client.get(ResourceType::Lock, None).await?;
    resp.resources()
        .into_iter()
        .map(Lock::from_resource)
        .collect()
}

/// Fetch a single lock by ID.
pub async fn fetch_lock(client: &mut AlarmClient, id: &str) -> Result<Lock> {
    let resp = client.get(ResourceType::Lock, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("lock {id} not found")))?;
    Lock::from_resource(resource)
}

/// Lock a lock device.
pub async fn lock(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(ResourceType::Lock, id, LockCommand::Lock.api_action(), body)
        .await?;
    Ok(())
}

/// Unlock a lock device.
pub async fn unlock(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(
            ResourceType::Lock,
            id,
            LockCommand::Unlock.api_action(),
            body,
        )
        .await?;
    Ok(())
}
