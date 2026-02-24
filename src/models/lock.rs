//! Lock model.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Lock state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockState {
    Unknown,
    Locked,
    Unlocked,
}

impl LockState {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => LockState::Locked,
            2 => LockState::Unlocked,
            _ => LockState::Unknown,
        }
    }
}

impl fmt::Display for LockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockState::Unknown => write!(f, "Unknown"),
            LockState::Locked => write!(f, "Locked"),
            LockState::Unlocked => write!(f, "Unlocked"),
        }
    }
}

/// Lock command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockCommand {
    Lock,
    Unlock,
}

impl LockCommand {
    pub fn api_action(&self) -> &'static str {
        match self {
            LockCommand::Lock => "lock",
            LockCommand::Unlock => "unlock",
        }
    }
}

/// Lock attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub desired_state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub low_battery: Option<bool>,
    #[serde(default)]
    pub critical_battery: Option<bool>,
    #[serde(default)]
    pub malfunction: Option<bool>,
}

/// A lock device.
#[derive(Debug, Clone)]
pub struct Lock {
    pub id: String,
    pub name: String,
    pub state: LockState,
    pub desired_state: LockState,
    pub low_battery: bool,
    pub malfunction: bool,
    pub attributes: LockAttributes,
}

impl Lock {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: LockAttributes =
            serde_json::from_value(resource.attributes.clone()).map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse lock attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Lock {}", resource.id));

        Ok(Lock {
            id: resource.id.clone(),
            name,
            state: LockState::from_code(attrs.state),
            desired_state: LockState::from_code(attrs.desired_state),
            low_battery: attrs.low_battery.unwrap_or(false),
            malfunction: attrs.malfunction.unwrap_or(false),
            attributes: attrs,
        })
    }
}

impl Device for Lock {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::Lock
    }
}
