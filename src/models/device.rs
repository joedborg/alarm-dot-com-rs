//! Shared device traits and base types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Resource types used in Alarm.com API paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Partition,
    Sensor,
    Lock,
    GarageDoor,
    Gate,
    Light,
    Thermostat,
    Camera,
    WaterSensor,
    WaterValve,
    System,
    ImageSensor,
}

impl ResourceType {
    /// The API path segment for this resource type (camelCase, pluralized).
    pub fn api_path(&self) -> &'static str {
        match self {
            ResourceType::Partition => "devices/partitions",
            ResourceType::Sensor => "devices/sensors",
            ResourceType::Lock => "devices/locks",
            ResourceType::GarageDoor => "devices/garageDoors",
            ResourceType::Gate => "devices/gates",
            ResourceType::Light => "devices/lights",
            ResourceType::Thermostat => "devices/thermostats",
            ResourceType::Camera => "devices/cameras",
            ResourceType::WaterSensor => "devices/waterSensors",
            ResourceType::WaterValve => "devices/waterValves",
            ResourceType::System => "systems/systems",
            ResourceType::ImageSensor => "devices/imageSensors",
        }
    }

    /// The JSON:API `type` string this resource uses.
    pub fn type_string(&self) -> &'static str {
        match self {
            ResourceType::Partition => "devices/partition",
            ResourceType::Sensor => "devices/sensor",
            ResourceType::Lock => "devices/lock",
            ResourceType::GarageDoor => "devices/garageDoor",
            ResourceType::Gate => "devices/gate",
            ResourceType::Light => "devices/light",
            ResourceType::Thermostat => "devices/thermostat",
            ResourceType::Camera => "devices/camera",
            ResourceType::WaterSensor => "devices/waterSensor",
            ResourceType::WaterValve => "devices/waterValve",
            ResourceType::System => "systems/system",
            ResourceType::ImageSensor => "devices/imageSensor",
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Common attributes shared by all managed devices.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseDeviceAttributes {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub has_permission: Option<bool>,
    #[serde(default)]
    pub can_be_renamed: Option<bool>,
    #[serde(default)]
    pub can_be_saved: Option<bool>,
    #[serde(default)]
    pub can_access_web_settings: Option<bool>,
    #[serde(default)]
    pub can_access_app_settings: Option<bool>,
    #[serde(default)]
    pub mac_address: Option<String>,
}

/// A trait for types that represent a device with a name and ID.
pub trait Device {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn resource_type() -> ResourceType;
}
