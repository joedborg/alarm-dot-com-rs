//! Example: Fetch and display Alarm.com system status.
//!
//! Usage:
//!   ALARM_USERNAME=you@email.com ALARM_PASSWORD=secret ALARM_MFA_COOKIE=cookie cargo run --example status
//!
//! To obtain the MFA cookie, log in to www.alarm.com in your browser, complete 2FA,
//! then copy the `twoFactorAuthenticationId` cookie value from your browser's dev tools.

use alarm_dot_com::AlarmDotCom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let username =
        std::env::var("ALARM_USERNAME").expect("Set ALARM_USERNAME environment variable");
    let password =
        std::env::var("ALARM_PASSWORD").expect("Set ALARM_PASSWORD environment variable");

    let mut alarm = AlarmDotCom::new(&username, &password);

    let mfa_cookie =
        std::env::var("ALARM_MFA_COOKIE").expect("Set ALARM_MFA_COOKIE environment variable");
    alarm.set_trusted_device_cookie(mfa_cookie);

    eprint!("Logging in...");
    alarm.login().await?;
    eprint!(" fetching status...");
    let status = alarm.fetch_status().await?;
    eprintln!(" done.\n");

    // Header
    let title = format!(" {} ", status.system.name);
    let bar = "=".repeat(title.len());
    println!("{bar}");
    println!("{title}");
    println!("{bar}\n");

    // Partitions
    if !status.partitions.is_empty() {
        section("PARTITIONS");
        for p in &status.partitions {
            let state_str = format!("{}", p.state);
            let mut tags = Vec::new();
            if p.has_active_alarm {
                tags.push("ALARM!");
            }
            print_row(&p.name, &state_str, &tags);
        }
        println!();
    }

    // Sensors
    if !status.sensors.is_empty() {
        section("SENSORS");
        for s in &status.sensors {
            let state_str = format!("{}", s.state);
            let mut tags = Vec::new();
            if s.is_bypassed {
                tags.push("bypassed");
            }
            if s.low_battery {
                tags.push("low bat");
            }
            if s.malfunction {
                tags.push("fault");
            }
            let label = format!("{} ({})", s.name, s.sensor_type);
            print_row(&label, &state_str, &tags);
        }
        println!();
    }

    // Locks
    if !status.locks.is_empty() {
        section("LOCKS");
        for l in &status.locks {
            let state_str = format!("{}", l.state);
            let mut tags = Vec::new();
            if l.low_battery {
                tags.push("low bat");
            }
            if l.malfunction {
                tags.push("fault");
            }
            print_row(&l.name, &state_str, &tags);
        }
        println!();
    }

    // Garage Doors
    if !status.garage_doors.is_empty() {
        section("GARAGE DOORS");
        for g in &status.garage_doors {
            let state_str = format!("{}", g.state);
            let mut tags = Vec::new();
            if g.malfunction {
                tags.push("fault");
            }
            print_row(&g.name, &state_str, &tags);
        }
        println!();
    }

    // Lights
    if !status.lights.is_empty() {
        section("LIGHTS");
        for l in &status.lights {
            let state_str = match l.brightness {
                Some(b) if l.is_dimmer => format!("{} ({b}%)", l.state),
                _ => format!("{}", l.state),
            };
            let mut tags = Vec::new();
            if l.malfunction {
                tags.push("fault");
            }
            print_row(&l.name, &state_str, &tags);
        }
        println!();
    }

    // Thermostats
    if !status.thermostats.is_empty() {
        section("THERMOSTATS");
        for t in &status.thermostats {
            let temp = t
                .ambient_temp
                .map(|v| format!("{v:.0}°F"))
                .unwrap_or_else(|| "--".to_string());
            let mode = format!("{}", t.mode);

            let mut detail_parts = vec![format!("{temp}  mode: {mode}")];

            if let Some(h) = t.heat_setpoint {
                detail_parts.push(format!("heat: {h:.0}°"));
            }
            if let Some(c) = t.cool_setpoint {
                detail_parts.push(format!("cool: {c:.0}°"));
            }
            if let Some(h) = t.humidity {
                detail_parts.push(format!("humidity: {h:.0}%"));
            }

            let detail = detail_parts.join("  ");
            let mut tags = Vec::new();
            if t.malfunction {
                tags.push("fault");
            }
            print_row(&t.name, &detail, &tags);
        }
        println!();
    }

    Ok(())
}

fn section(name: &str) {
    println!("{name}");
    println!("{}", "-".repeat(name.len()));
}

fn print_row(name: &str, value: &str, tags: &[&str]) {
    let tag_str = if tags.is_empty() {
        String::new()
    } else {
        format!("  [{}]", tags.join(", "))
    };
    println!("  {name:<30} {value}{tag_str}");
}
