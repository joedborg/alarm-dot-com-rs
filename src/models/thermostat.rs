//! Thermostat model.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Thermostat mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermostatMode {
    Unknown,
    Off,
    Heat,
    Cool,
    Auto,
    AuxHeat,
}

impl ThermostatMode {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => ThermostatMode::Off,
            2 => ThermostatMode::Heat,
            3 => ThermostatMode::Cool,
            4 => ThermostatMode::Auto,
            5 => ThermostatMode::AuxHeat,
            _ => ThermostatMode::Unknown,
        }
    }
}

impl fmt::Display for ThermostatMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThermostatMode::Unknown => write!(f, "Unknown"),
            ThermostatMode::Off => write!(f, "Off"),
            ThermostatMode::Heat => write!(f, "Heat"),
            ThermostatMode::Cool => write!(f, "Cool"),
            ThermostatMode::Auto => write!(f, "Auto"),
            ThermostatMode::AuxHeat => write!(f, "Aux Heat"),
        }
    }
}

/// Thermostat fan mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FanMode {
    Unknown,
    Auto,
    On,
    Circulate,
}

impl FanMode {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => FanMode::Auto,
            2 => FanMode::On,
            3 => FanMode::Circulate,
            _ => FanMode::Unknown,
        }
    }
}

impl fmt::Display for FanMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FanMode::Unknown => write!(f, "Unknown"),
            FanMode::Auto => write!(f, "Auto"),
            FanMode::On => write!(f, "On"),
            FanMode::Circulate => write!(f, "Circulate"),
        }
    }
}

/// Thermostat attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThermostatAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub desired_state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub ambient_temp: Option<f64>,
    #[serde(default)]
    pub heat_setpoint: Option<f64>,
    #[serde(default)]
    pub cool_setpoint: Option<f64>,
    #[serde(default)]
    pub fan_mode: Option<u32>,
    #[serde(default)]
    pub humidity: Option<f64>,
    #[serde(default)]
    pub supports_fan_mode: Option<bool>,
    #[serde(default)]
    pub supports_humidity: Option<bool>,
    #[serde(default)]
    pub malfunction: Option<bool>,
    #[serde(default)]
    pub supports_auto_mode: Option<bool>,
}

/// A thermostat device.
#[derive(Debug, Clone)]
pub struct Thermostat {
    pub id: String,
    pub name: String,
    pub mode: ThermostatMode,
    pub ambient_temp: Option<f64>,
    pub heat_setpoint: Option<f64>,
    pub cool_setpoint: Option<f64>,
    pub fan_mode: FanMode,
    pub humidity: Option<f64>,
    pub malfunction: bool,
    pub attributes: ThermostatAttributes,
}

impl Thermostat {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: ThermostatAttributes = serde_json::from_value(resource.attributes.clone())
            .map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse thermostat attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Thermostat {}", resource.id));

        Ok(Thermostat {
            id: resource.id.clone(),
            name,
            mode: ThermostatMode::from_code(attrs.state),
            ambient_temp: attrs.ambient_temp,
            heat_setpoint: attrs.heat_setpoint,
            cool_setpoint: attrs.cool_setpoint,
            fan_mode: FanMode::from_code(attrs.fan_mode.unwrap_or(0)),
            humidity: attrs.humidity,
            malfunction: attrs.malfunction.unwrap_or(false),
            attributes: attrs,
        })
    }
}

impl Device for Thermostat {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::Thermostat
    }
}
