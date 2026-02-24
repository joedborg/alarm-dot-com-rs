//! Partition (alarm panel) model.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Alarm panel partition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionState {
    Unknown,
    Disarmed,
    ArmedStay,
    ArmedAway,
    ArmedNight,
    Hidden,
}

impl PartitionState {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => PartitionState::Disarmed,
            2 => PartitionState::ArmedStay,
            3 => PartitionState::ArmedAway,
            4 => PartitionState::ArmedNight,
            5 => PartitionState::Hidden,
            _ => PartitionState::Unknown,
        }
    }

    pub fn is_armed(&self) -> bool {
        matches!(
            self,
            PartitionState::ArmedStay
                | PartitionState::ArmedAway
                | PartitionState::ArmedNight
                | PartitionState::Hidden
        )
    }
}

impl fmt::Display for PartitionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartitionState::Unknown => write!(f, "Unknown"),
            PartitionState::Disarmed => write!(f, "Disarmed"),
            PartitionState::ArmedStay => write!(f, "Armed Stay"),
            PartitionState::ArmedAway => write!(f, "Armed Away"),
            PartitionState::ArmedNight => write!(f, "Armed Night"),
            PartitionState::Hidden => write!(f, "Hidden"),
        }
    }
}

/// Extended arming options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmingOption {
    BypassSensors,
    NoEntryDelay,
    SilentArming,
    NightArming,
    SelectivelyBypassSensors,
    ForceArm,
}

impl ArmingOption {
    pub fn api_key(&self) -> &'static str {
        match self {
            ArmingOption::BypassSensors => "forceBypass",
            ArmingOption::NoEntryDelay => "noEntryDelay",
            ArmingOption::SilentArming => "silentArming",
            ArmingOption::NightArming => "nightArming",
            ArmingOption::SelectivelyBypassSensors => "selectiveBypass",
            ArmingOption::ForceArm => "forceArm",
        }
    }
}

/// Partition command to send to the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionCommand {
    Disarm,
    ArmStay,
    ArmAway,
}

impl PartitionCommand {
    pub fn api_action(&self) -> &'static str {
        match self {
            PartitionCommand::Disarm => "disarm",
            PartitionCommand::ArmStay => "armStay",
            PartitionCommand::ArmAway => "armAway",
        }
    }
}

/// Alarm.com partition attributes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub desired_state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub partition_id: Option<String>,
    #[serde(default)]
    pub has_active_alarm: Option<bool>,
    #[serde(default)]
    pub needs_clear_issues_prompt: Option<bool>,
    #[serde(default)]
    pub has_open_bypassable_sensors: Option<bool>,
    #[serde(default)]
    pub has_sensor_in_trouble_condition: Option<bool>,
    #[serde(default)]
    pub can_bypass_sensor_when_armed: Option<bool>,
}

/// A partition (alarm panel zone).
#[derive(Debug, Clone)]
pub struct Partition {
    pub id: String,
    pub name: String,
    pub state: PartitionState,
    pub desired_state: PartitionState,
    pub has_active_alarm: bool,
    pub attributes: PartitionAttributes,
    /// IDs of child devices (sensors, etc.) in this partition.
    pub child_device_ids: Vec<String>,
}

impl Partition {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: PartitionAttributes = serde_json::from_value(resource.attributes.clone())
            .map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse partition attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Partition {}", resource.id));

        // Collect child device IDs from relationships
        let mut child_ids = Vec::new();
        for key in resource.relationships.keys() {
            if key != "system" {
                child_ids.extend(resource.related_ids(key));
            }
        }

        Ok(Partition {
            id: resource.id.clone(),
            name,
            state: PartitionState::from_code(attrs.state),
            desired_state: PartitionState::from_code(attrs.desired_state),
            has_active_alarm: attrs.has_active_alarm.unwrap_or(false),
            attributes: attrs,
            child_device_ids: child_ids,
        })
    }
}

impl Device for Partition {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::Partition
    }
}

/// Build the JSON body for an arm/disarm command.
pub fn build_arming_body(options: &[ArmingOption]) -> serde_json::Value {
    let mut body = serde_json::json!({
        "forceBypass": false,
        "noEntryDelay": false,
        "silentArming": false,
        "nightArming": false,
        "selectiveBypass": false,
        "forceArm": false,
        "statePollOnly": false,
    });

    for opt in options {
        body[opt.api_key()] = serde_json::Value::Bool(true);
    }

    body
}
