//! Light model.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Light state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightState {
    Unknown,
    On,
    Off,
}

impl LightState {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => LightState::On,
            2 => LightState::Off,
            _ => LightState::Unknown,
        }
    }
}

impl fmt::Display for LightState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LightState::Unknown => write!(f, "Unknown"),
            LightState::On => write!(f, "On"),
            LightState::Off => write!(f, "Off"),
        }
    }
}

/// Light command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightCommand {
    TurnOn,
    TurnOff,
}

impl LightCommand {
    pub fn api_action(&self) -> &'static str {
        match self {
            LightCommand::TurnOn => "turnOn",
            LightCommand::TurnOff => "turnOff",
        }
    }
}

/// Light attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub desired_state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub light_level: Option<u32>,
    #[serde(default)]
    pub is_dimmer: Option<bool>,
    #[serde(default)]
    pub malfunction: Option<bool>,
}

/// A light device.
#[derive(Debug, Clone)]
pub struct Light {
    pub id: String,
    pub name: String,
    pub state: LightState,
    pub desired_state: LightState,
    pub brightness: Option<u32>,
    pub is_dimmer: bool,
    pub malfunction: bool,
    pub attributes: LightAttributes,
}

impl Light {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: LightAttributes =
            serde_json::from_value(resource.attributes.clone()).map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse light attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Light {}", resource.id));

        Ok(Light {
            id: resource.id.clone(),
            name,
            state: LightState::from_code(attrs.state),
            desired_state: LightState::from_code(attrs.desired_state),
            brightness: attrs.light_level,
            is_dimmer: attrs.is_dimmer.unwrap_or(false),
            malfunction: attrs.malfunction.unwrap_or(false),
            attributes: attrs,
        })
    }
}

impl Device for Light {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::Light
    }
}
