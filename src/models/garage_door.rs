//! Garage door model.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Garage door state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GarageDoorState {
    Unknown,
    Open,
    Closed,
    Opening,
    Closing,
}

impl GarageDoorState {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => GarageDoorState::Open,
            2 => GarageDoorState::Closed,
            3 => GarageDoorState::Opening,
            4 => GarageDoorState::Closing,
            _ => GarageDoorState::Unknown,
        }
    }
}

impl fmt::Display for GarageDoorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GarageDoorState::Unknown => write!(f, "Unknown"),
            GarageDoorState::Open => write!(f, "Open"),
            GarageDoorState::Closed => write!(f, "Closed"),
            GarageDoorState::Opening => write!(f, "Opening"),
            GarageDoorState::Closing => write!(f, "Closing"),
        }
    }
}

/// Garage door command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageDoorCommand {
    Open,
    Close,
}

impl GarageDoorCommand {
    pub fn api_action(&self) -> &'static str {
        match self {
            GarageDoorCommand::Open => "open",
            GarageDoorCommand::Close => "close",
        }
    }
}

/// Garage door attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GarageDoorAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub desired_state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub malfunction: Option<bool>,
}

/// A garage door device.
#[derive(Debug, Clone)]
pub struct GarageDoor {
    pub id: String,
    pub name: String,
    pub state: GarageDoorState,
    pub desired_state: GarageDoorState,
    pub malfunction: bool,
    pub attributes: GarageDoorAttributes,
}

impl GarageDoor {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: GarageDoorAttributes = serde_json::from_value(resource.attributes.clone())
            .map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse garage door attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Garage Door {}", resource.id));

        Ok(GarageDoor {
            id: resource.id.clone(),
            name,
            state: GarageDoorState::from_code(attrs.state),
            desired_state: GarageDoorState::from_code(attrs.desired_state),
            malfunction: attrs.malfunction.unwrap_or(false),
            attributes: attrs,
        })
    }
}

impl Device for GarageDoor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::GarageDoor
    }
}
