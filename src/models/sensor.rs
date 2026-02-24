//! Sensor model (door/window contacts, motion, glass break, smoke, CO, flood).

use serde::{Deserialize, Serialize};
use std::fmt;

use super::device::{Device, ResourceType};
use super::jsonapi::Resource;
use crate::error::{AlarmError, Result};

/// Sensor state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorState {
    Unknown,
    Closed,
    Open,
    Idle,
    Active,
    Dry,
    Wet,
}

impl SensorState {
    pub fn from_code(code: u32) -> Self {
        match code {
            0 => SensorState::Unknown,
            1 => SensorState::Closed,
            2 => SensorState::Open,
            3 => SensorState::Idle,
            4 => SensorState::Active,
            5 => SensorState::Dry,
            6 => SensorState::Wet,
            _ => SensorState::Unknown,
        }
    }
}

impl fmt::Display for SensorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorState::Unknown => write!(f, "Unknown"),
            SensorState::Closed => write!(f, "Closed"),
            SensorState::Open => write!(f, "Open"),
            SensorState::Idle => write!(f, "Idle"),
            SensorState::Active => write!(f, "Active"),
            SensorState::Dry => write!(f, "Dry"),
            SensorState::Wet => write!(f, "Wet"),
        }
    }
}

/// Type of sensor device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
    DoorWindow,
    Motion,
    GlassBreak,
    Smoke,
    CarbonMonoxide,
    Flood,
    Temperature,
    Freeze,
    Shock,
    Tilt,
    Other,
}

impl fmt::Display for SensorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorType::DoorWindow => write!(f, "Door/Window"),
            SensorType::Motion => write!(f, "Motion"),
            SensorType::GlassBreak => write!(f, "Glass Break"),
            SensorType::Smoke => write!(f, "Smoke"),
            SensorType::CarbonMonoxide => write!(f, "CO"),
            SensorType::Flood => write!(f, "Flood"),
            SensorType::Temperature => write!(f, "Temperature"),
            SensorType::Freeze => write!(f, "Freeze"),
            SensorType::Shock => write!(f, "Shock"),
            SensorType::Tilt => write!(f, "Tilt"),
            SensorType::Other => write!(f, "Other"),
        }
    }
}

/// Sensor attributes from the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorAttributes {
    #[serde(default)]
    pub state: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub device_type: Option<u32>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub can_be_bypassed: Option<bool>,
    #[serde(default)]
    pub is_bypassed: Option<bool>,
    #[serde(default)]
    pub low_battery: Option<bool>,
    #[serde(default)]
    pub critical_battery: Option<bool>,
    #[serde(default)]
    pub malfunction: Option<bool>,
}

/// A sensor device.
#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub name: String,
    pub state: SensorState,
    pub sensor_type: SensorType,
    pub is_bypassed: bool,
    pub low_battery: bool,
    pub malfunction: bool,
    pub attributes: SensorAttributes,
}

impl Sensor {
    pub fn from_resource(resource: &Resource) -> Result<Self> {
        let attrs: SensorAttributes =
            serde_json::from_value(resource.attributes.clone()).map_err(|e| {
                AlarmError::UnexpectedResponse(format!("failed to parse sensor attributes: {e}"))
            })?;

        let name = attrs
            .description
            .clone()
            .unwrap_or_else(|| format!("Sensor {}", resource.id));

        let sensor_type = match attrs.device_type {
            Some(1) => SensorType::DoorWindow,
            Some(2) => SensorType::Motion,
            Some(5) => SensorType::GlassBreak,
            Some(6) => SensorType::Smoke,
            Some(8) => SensorType::CarbonMonoxide,
            Some(9) => SensorType::Flood,
            Some(10) => SensorType::Temperature,
            Some(14) => SensorType::Tilt,
            Some(16) => SensorType::Shock,
            Some(19) => SensorType::Freeze,
            _ => SensorType::Other,
        };

        Ok(Sensor {
            id: resource.id.clone(),
            name,
            state: SensorState::from_code(attrs.state),
            sensor_type,
            is_bypassed: attrs.is_bypassed.unwrap_or(false),
            low_battery: attrs.low_battery.unwrap_or(false),
            malfunction: attrs.malfunction.unwrap_or(false),
            attributes: attrs,
        })
    }
}

impl Device for Sensor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn resource_type() -> ResourceType {
        ResourceType::Sensor
    }
}
