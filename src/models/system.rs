//! System model.

use serde::{Deserialize, Serialize};

use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// System attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemAttributes {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_snapshots: Option<bool>,
    #[serde(default)]
    pub supports_secure_arming: Option<bool>,
    #[serde(default)]
    pub remain_logged_in: Option<bool>,
}

/// An Alarm.com system.
#[derive(Debug, Clone)]
pub struct System {
    pub id: String,
    pub name: String,
    /// IDs of partitions in this system.
    pub partition_ids: Vec<String>,
    /// IDs of sensors in this system.
    pub sensor_ids: Vec<String>,
    /// IDs of locks in this system.
    pub lock_ids: Vec<String>,
    /// IDs of garage doors in this system.
    pub garage_door_ids: Vec<String>,
    /// IDs of lights in this system.
    pub light_ids: Vec<String>,
    /// IDs of thermostats in this system.
    pub thermostat_ids: Vec<String>,
    pub attributes: SystemAttributes,
}

impl System {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: SystemAttributes =
            serde_json::from_value(resource.attributes.clone()).map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse system attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("System {}", resource.id));

        Ok(System {
            id: resource.id.clone(),
            name,
            partition_ids: resource.related_ids("partitions"),
            sensor_ids: resource.related_ids("sensors"),
            lock_ids: resource.related_ids("locks"),
            garage_door_ids: resource.related_ids("garageDoors"),
            light_ids: resource.related_ids("lights"),
            thermostat_ids: resource.related_ids("thermostats"),
            attributes: attrs,
        })
    }
}
