# alarm-dot-com

A Rust library for interfacing with [Alarm.com](https://www.alarm.com/) security panels.

This library reverse-engineers the Alarm.com web portal's AJAX API to provide programmatic access to your alarm panel, sensors, locks, garage doors, lights, and thermostats.

> **Note:** ADT Control uses Alarm.com as its backend platform, so this library is fully compatible with ADT Control systems.

> **Disclaimer:** This is an unofficial library, not affiliated with Alarm.com or ADT. The underlying API is undocumented and may change without notice.

## Features

- **Partitions** -- arm stay, arm away, arm night, disarm, clear faults
- **Sensors** -- door/window, motion, glass break, smoke, CO, flood, temperature
- **Locks** -- lock, unlock, state monitoring
- **Garage doors** -- open, close, state monitoring
- **Lights** -- on, off, brightness control for dimmers
- **Thermostats** -- mode, heat/cool setpoints, ambient temperature and humidity
- **Session management** -- automatic retry on transient failures, re-login on expired sessions

## Prerequisites

This library requires a **trusted device cookie** to bypass Alarm.com's two-factor authentication. Follow these steps to obtain it:

1. Go to [www.alarm.com](https://www.alarm.com) and log in with your credentials
2. Complete the 2FA challenge when prompted (you may be asked to name the device — this can be anything)
3. Open your browser's developer tools:
   - **Chrome / Edge:** right-click the page → _Inspect_ → _Application_ tab → _Cookies_ → `https://www.alarm.com`
   - **Firefox:** right-click → _Inspect_ → _Storage_ tab → _Cookies_ → `https://www.alarm.com`
   - **Safari:** _Develop_ menu → _Show Web Inspector_ → _Storage_ tab → _Cookies_
4. Find the cookie named `twoFactorAuthenticationId` — its value is a 64-character alphanumeric string
5. Copy that value and set it as your `ALARM_MFA_COOKIE` environment variable

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
alarm-dot-com = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use alarm_dot_com::AlarmDotCom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut alarm = AlarmDotCom::new("user@example.com", "password");
    alarm.set_trusted_device_cookie("your-mfa-cookie-value");

    alarm.login().await?;

    let status = alarm.fetch_status().await?;
    println!("System: {}", status.system.name);

    for p in &status.partitions {
        println!("  Partition {}: {}", p.name, p.state);
    }
    for s in &status.sensors {
        println!("  Sensor {} ({}): {}", s.name, s.sensor_type, s.state);
    }

    // Arm in stay mode
    if let Some(partition) = status.partitions.first() {
        alarm.arm_stay(&partition.id, &[]).await?;
    }

    Ok(())
}
```

## CLI Examples

Set your credentials as environment variables, then run the examples:

```sh
export ALARM_USERNAME="you@email.com"
export ALARM_PASSWORD="your-password"
export ALARM_MFA_COOKIE="your-twoFactorAuthenticationId-cookie"
```

### Get system status

```sh
cargo run --example status
```

Prints all partitions, sensors, locks, garage doors, lights, and thermostats with their current states.

### Arm/disarm

```sh
# Arm stay
cargo run --example arm -- stay

# Arm away
cargo run --example arm -- away

# Arm night
cargo run --example arm -- night

# Disarm
cargo run --example arm -- disarm

# Arm stay, bypassing open sensors
cargo run --example arm -- stay-bypass

# Target a specific partition
cargo run --example arm -- stay PARTITION_ID
```

## API Overview

### `AlarmDotCom`

The main entry point. Created with credentials, then `login()` to authenticate.

| Method                                           | Description                                                                      |
| ------------------------------------------------ | -------------------------------------------------------------------------------- |
| `set_trusted_device_cookie(cookie)`              | Set the MFA bypass cookie (required for 2FA accounts).                           |
| `login()`                                        | Authenticate. Returns `Err(TwoFactorRequired)` if MFA cookie is missing/invalid. |
| `keep_alive()`                                   | Prevent session timeout. Returns `false` if expired.                             |
| `fetch_status()`                                 | Fetch full system snapshot (all device types).                                   |
| `fetch_partitions()`                             | Fetch alarm panel partitions.                                                    |
| `arm_stay(id, opts)`                             | Arm a partition in Stay mode.                                                    |
| `arm_away(id, opts)`                             | Arm a partition in Away mode.                                                    |
| `arm_night(id, opts)`                            | Arm a partition in Night mode.                                                   |
| `disarm(id)`                                     | Disarm a partition.                                                              |
| `fetch_sensors()`                                | Fetch all sensors.                                                               |
| `fetch_locks()`                                  | Fetch all locks.                                                                 |
| `lock(id)` / `unlock(id)`                        | Lock or unlock a lock.                                                           |
| `fetch_garage_doors()`                           | Fetch all garage doors.                                                          |
| `open_garage_door(id)` / `close_garage_door(id)` | Open or close a garage door.                                                     |
| `fetch_lights()`                                 | Fetch all lights.                                                                |
| `turn_on_light(id)` / `turn_off_light(id)`       | Turn a light on or off.                                                          |
| `set_light_brightness(id, level)`                | Set dimmer brightness (0-100).                                                   |
| `fetch_thermostats()`                            | Fetch all thermostats.                                                           |
| `set_thermostat_mode(id, mode)`                  | Set thermostat mode (off/heat/cool/auto).                                        |
| `set_heat_setpoint(id, temp)`                    | Set heat target temperature.                                                     |
| `set_cool_setpoint(id, temp)`                    | Set cool target temperature.                                                     |

### Arming Options

Pass to `arm_stay`, `arm_away`, or `arm_night`:

```rust
use alarm_dot_com::models::partition::ArmingOption;

alarm.arm_away(&id, &[
    ArmingOption::BypassSensors,
    ArmingOption::SilentArming,
]).await?;
```

| Option          | Description                                         |
| --------------- | --------------------------------------------------- |
| `BypassSensors` | Force bypass all open zones.                        |
| `NoEntryDelay`  | Sound alarm immediately on entry zone trigger.      |
| `SilentArming`  | Suppress arming/exit delay tones at the panel.      |
| `NightArming`   | Enable night arming (auto-included by `arm_night`). |
| `ForceArm`      | Force arm regardless of faults.                     |

## Error Handling

All operations return `Result<T, AlarmError>`. Key variants:

| Error                  | Meaning                                                  |
| ---------------------- | -------------------------------------------------------- |
| `TwoFactorRequired`    | MFA cookie missing or invalid -- set `ALARM_MFA_COOKIE`. |
| `AuthenticationFailed` | Bad credentials.                                         |
| `SessionExpired`       | Session timed out -- re-login needed.                    |
| `ServiceUnavailable`   | Alarm.com unreachable or returned server error.          |
| `NotAuthorized`        | Account lacks permission for this operation.             |
| `UnknownDevice`        | Device ID not found.                                     |

## Project Structure

```
src/
  lib.rs              High-level AlarmDotCom API
  auth.rs             Login flow, ViewState parsing
  client.rs           HTTP client, cookie jar, retry logic
  error.rs            Error types
  models/
    jsonapi.rs        JSON:API response deserialization
    device.rs         Shared traits and ResourceType enum
    partition.rs      Alarm panel state and commands
    sensor.rs         Sensor types and state
    lock.rs           Lock state and commands
    garage_door.rs    Garage door state and commands
    light.rs          Light state and commands
    thermostat.rs     Thermostat state and commands
    system.rs         System-level info
  controllers/
    partitions.rs     Partition fetch/command operations
    sensors.rs        Sensor fetch operations
    locks.rs          Lock fetch/command operations
    garage_doors.rs   Garage door fetch/command operations
    lights.rs         Light fetch/command operations
    thermostats.rs    Thermostat fetch/command operations
    systems.rs        System discovery and fetch
examples/
  status.rs           Print full system status
  arm.rs              Arm/disarm via CLI
```

## License

MIT
