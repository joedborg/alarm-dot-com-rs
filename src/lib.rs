//! # alarm-dot-com
//!
//! A Rust library for interfacing with Alarm.com security panels.
//!
//! This library reverse-engineers the Alarm.com web portal's AJAX API to provide
//! programmatic access to your alarm panel, sensors, locks, garage doors, lights,
//! and thermostats.
//!
//! ## Quick Start
//!
//! ```no_run
//! use alarm_dot_com::AlarmDotCom;
//!
//! #[tokio::main]
//! async fn main() -> alarm_dot_com::error::Result<()> {
//!     let mut alarm = AlarmDotCom::new("user@example.com", "password");
//!
//!     // For accounts with 2FA, set the trusted device cookie from your browser:
//!     // alarm.set_trusted_device_cookie("cookie-value-from-browser");
//!
//!     alarm.login().await?;
//!
//!     let status = alarm.fetch_status().await?;
//!     for partition in &status.partitions {
//!         println!("{}: {}", partition.name, partition.state);
//!     }
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod controllers;
pub mod error;
pub mod models;

use client::AlarmClient;
use error::Result;
use models::garage_door::GarageDoor;
use models::light::Light;
use models::lock::Lock;
use models::partition::{ArmingOption, Partition};
use models::sensor::Sensor;
use models::system::System;
use models::thermostat::Thermostat;

/// A snapshot of the full system state.
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub system: System,
    pub partitions: Vec<Partition>,
    pub sensors: Vec<Sensor>,
    pub locks: Vec<Lock>,
    pub garage_doors: Vec<GarageDoor>,
    pub lights: Vec<Light>,
    pub thermostats: Vec<Thermostat>,
}

/// High-level API entry point for Alarm.com.
///
/// Wraps authentication, session management, and all device controllers
/// behind a single ergonomic interface.
pub struct AlarmDotCom {
    client: AlarmClient,
}

impl AlarmDotCom {
    /// Create a new `AlarmDotCom` instance with the given credentials.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        AlarmDotCom {
            client: AlarmClient::new(username, password),
        }
    }

    /// Set a trusted device cookie to bypass 2FA on login.
    ///
    /// This is the `twoFactorAuthenticationId` cookie value from your browser's
    /// cookies for `www.alarm.com` after completing 2FA manually. With this set,
    /// the server recognizes the client as a trusted device and skips 2FA.
    pub fn set_trusted_device_cookie(&mut self, cookie: impl Into<String>) {
        self.client.set_trusted_device_cookie(cookie);
    }

    /// Get the current trusted device cookie value, if available.
    pub fn trusted_device_cookie(&self) -> Option<&str> {
        self.client.trusted_device_cookie()
    }

    /// Perform the login flow.
    ///
    /// Returns `Err(AlarmError::TwoFactorRequired)` if 2FA is needed and no
    /// trusted device cookie was provided via [`set_trusted_device_cookie`](Self::set_trusted_device_cookie).
    pub async fn login(&mut self) -> Result<()> {
        self.client.login().await
    }

    /// Whether the client is currently authenticated.
    pub fn is_logged_in(&self) -> bool {
        self.client.is_logged_in()
    }

    /// Send a keep-alive signal to prevent session timeout.
    /// Returns `false` if the session has expired (re-login needed).
    pub async fn keep_alive(&mut self) -> Result<bool> {
        self.client.keep_alive().await
    }

    // ---- System ----

    /// Fetch the primary system and all its devices in one call.
    pub async fn fetch_status(&mut self) -> Result<SystemStatus> {
        let system = controllers::systems::fetch_primary_system(&mut self.client).await?;

        let partitions = controllers::partitions::fetch_partitions(&mut self.client)
            .await
            .unwrap_or_default();
        let sensors = controllers::sensors::fetch_sensors(&mut self.client)
            .await
            .unwrap_or_default();
        let locks = controllers::locks::fetch_locks(&mut self.client)
            .await
            .unwrap_or_default();
        let garage_doors = controllers::garage_doors::fetch_garage_doors(&mut self.client)
            .await
            .unwrap_or_default();
        let lights = controllers::lights::fetch_lights(&mut self.client)
            .await
            .unwrap_or_default();
        let thermostats = controllers::thermostats::fetch_thermostats(&mut self.client)
            .await
            .unwrap_or_default();

        Ok(SystemStatus {
            system,
            partitions,
            sensors,
            locks,
            garage_doors,
            lights,
            thermostats,
        })
    }

    /// Fetch the primary system info.
    pub async fn fetch_system(&mut self) -> Result<System> {
        controllers::systems::fetch_primary_system(&mut self.client).await
    }

    // ---- Partitions ----

    /// Fetch all partitions (alarm panels).
    pub async fn fetch_partitions(&mut self) -> Result<Vec<Partition>> {
        controllers::partitions::fetch_partitions(&mut self.client).await
    }

    /// Arm a partition in Stay mode.
    pub async fn arm_stay(&mut self, partition_id: &str, options: &[ArmingOption]) -> Result<()> {
        controllers::partitions::arm_stay(&mut self.client, partition_id, options).await
    }

    /// Arm a partition in Away mode.
    pub async fn arm_away(&mut self, partition_id: &str, options: &[ArmingOption]) -> Result<()> {
        controllers::partitions::arm_away(&mut self.client, partition_id, options).await
    }

    /// Arm a partition in Night mode.
    pub async fn arm_night(&mut self, partition_id: &str, options: &[ArmingOption]) -> Result<()> {
        controllers::partitions::arm_night(&mut self.client, partition_id, options).await
    }

    /// Disarm a partition.
    pub async fn disarm(&mut self, partition_id: &str) -> Result<()> {
        controllers::partitions::disarm(&mut self.client, partition_id).await
    }

    /// Clear faults on a partition.
    pub async fn clear_faults(&mut self, partition_id: &str) -> Result<()> {
        controllers::partitions::clear_faults(&mut self.client, partition_id).await
    }

    // ---- Sensors ----

    /// Fetch all sensors.
    pub async fn fetch_sensors(&mut self) -> Result<Vec<Sensor>> {
        controllers::sensors::fetch_sensors(&mut self.client).await
    }

    // ---- Locks ----

    /// Fetch all locks.
    pub async fn fetch_locks(&mut self) -> Result<Vec<Lock>> {
        controllers::locks::fetch_locks(&mut self.client).await
    }

    /// Lock a lock.
    pub async fn lock(&mut self, lock_id: &str) -> Result<()> {
        controllers::locks::lock(&mut self.client, lock_id).await
    }

    /// Unlock a lock.
    pub async fn unlock(&mut self, lock_id: &str) -> Result<()> {
        controllers::locks::unlock(&mut self.client, lock_id).await
    }

    // ---- Garage Doors ----

    /// Fetch all garage doors.
    pub async fn fetch_garage_doors(&mut self) -> Result<Vec<GarageDoor>> {
        controllers::garage_doors::fetch_garage_doors(&mut self.client).await
    }

    /// Open a garage door.
    pub async fn open_garage_door(&mut self, id: &str) -> Result<()> {
        controllers::garage_doors::open(&mut self.client, id).await
    }

    /// Close a garage door.
    pub async fn close_garage_door(&mut self, id: &str) -> Result<()> {
        controllers::garage_doors::close(&mut self.client, id).await
    }

    // ---- Lights ----

    /// Fetch all lights.
    pub async fn fetch_lights(&mut self) -> Result<Vec<Light>> {
        controllers::lights::fetch_lights(&mut self.client).await
    }

    /// Turn a light on.
    pub async fn turn_on_light(&mut self, id: &str) -> Result<()> {
        controllers::lights::turn_on(&mut self.client, id).await
    }

    /// Turn a light off.
    pub async fn turn_off_light(&mut self, id: &str) -> Result<()> {
        controllers::lights::turn_off(&mut self.client, id).await
    }

    /// Set a light's brightness (0-100).
    pub async fn set_light_brightness(&mut self, id: &str, level: u32) -> Result<()> {
        controllers::lights::set_brightness(&mut self.client, id, level).await
    }

    // ---- Thermostats ----

    /// Fetch all thermostats.
    pub async fn fetch_thermostats(&mut self) -> Result<Vec<Thermostat>> {
        controllers::thermostats::fetch_thermostats(&mut self.client).await
    }

    /// Set thermostat mode.
    pub async fn set_thermostat_mode(
        &mut self,
        id: &str,
        mode: models::thermostat::ThermostatMode,
    ) -> Result<()> {
        controllers::thermostats::set_mode(&mut self.client, id, mode).await
    }

    /// Set thermostat heat setpoint.
    pub async fn set_heat_setpoint(&mut self, id: &str, temp: f64) -> Result<()> {
        controllers::thermostats::set_heat_setpoint(&mut self.client, id, temp).await
    }

    /// Set thermostat cool setpoint.
    pub async fn set_cool_setpoint(&mut self, id: &str, temp: f64) -> Result<()> {
        controllers::thermostats::set_cool_setpoint(&mut self.client, id, temp).await
    }
}
