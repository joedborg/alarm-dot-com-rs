//! Partition controller -- fetch panel status, arm/disarm.

use crate::client::AlarmClient;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::partition::{build_arming_body, ArmingOption, Partition, PartitionCommand};

/// Fetch all partitions.
pub async fn fetch_partitions(client: &mut AlarmClient) -> Result<Vec<Partition>> {
    let resp = client.get(ResourceType::Partition, None).await?;
    resp.resources()
        .into_iter()
        .map(Partition::from_resource)
        .collect()
}

/// Fetch a single partition by ID.
pub async fn fetch_partition(client: &mut AlarmClient, id: &str) -> Result<Partition> {
    let resp = client.get(ResourceType::Partition, Some(id)).await?;
    let resource = resp
        .resources()
        .into_iter()
        .next()
        .ok_or_else(|| AlarmError::UnknownDevice(format!("partition {id} not found")))?;
    Partition::from_resource(resource)
}

/// Arm a partition in Stay mode.
pub async fn arm_stay(client: &mut AlarmClient, id: &str, options: &[ArmingOption]) -> Result<()> {
    send_partition_command(client, id, PartitionCommand::ArmStay, options).await
}

/// Arm a partition in Away mode.
pub async fn arm_away(client: &mut AlarmClient, id: &str, options: &[ArmingOption]) -> Result<()> {
    send_partition_command(client, id, PartitionCommand::ArmAway, options).await
}

/// Arm a partition in Night mode (arm stay + night arming option).
pub async fn arm_night(client: &mut AlarmClient, id: &str, options: &[ArmingOption]) -> Result<()> {
    let mut opts = options.to_vec();
    if !opts.contains(&ArmingOption::NightArming) {
        opts.push(ArmingOption::NightArming);
    }
    send_partition_command(client, id, PartitionCommand::ArmStay, &opts).await
}

/// Disarm a partition.
pub async fn disarm(client: &mut AlarmClient, id: &str) -> Result<()> {
    send_partition_command(client, id, PartitionCommand::Disarm, &[]).await
}

/// Clear faults on a partition.
pub async fn clear_faults(client: &mut AlarmClient, id: &str) -> Result<()> {
    let body = serde_json::json!({"statePollOnly": false});
    client
        .post(ResourceType::Partition, id, "clearIssues", body)
        .await?;
    Ok(())
}

async fn send_partition_command(
    client: &mut AlarmClient,
    id: &str,
    command: PartitionCommand,
    options: &[ArmingOption],
) -> Result<()> {
    let body = build_arming_body(options);
    client
        .post(ResourceType::Partition, id, command.api_action(), body)
        .await?;
    Ok(())
}
