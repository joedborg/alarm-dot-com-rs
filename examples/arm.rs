//! Example: Arm/disarm your Alarm.com panel.
//!
//! Usage:
//!   ALARM_USERNAME=you@email.com ALARM_PASSWORD=secret cargo run --example arm -- <command> [partition_id]
//!
//! Commands: stay, away, night, disarm
//!
//! If partition_id is not specified, the first partition is used.
//! ALARM_MFA_COOKIE is required — set it to your browser's
//! `twoFactorAuthenticationId` cookie value from www.alarm.com.

use alarm_dot_com::models::partition::ArmingOption;
use alarm_dot_com::AlarmDotCom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <stay|away|night|disarm> [partition_id]", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let partition_id_arg = args.get(2).cloned();

    let username =
        std::env::var("ALARM_USERNAME").expect("Set ALARM_USERNAME environment variable");
    let password =
        std::env::var("ALARM_PASSWORD").expect("Set ALARM_PASSWORD environment variable");

    let mut alarm = AlarmDotCom::new(&username, &password);

    let mfa_cookie =
        std::env::var("ALARM_MFA_COOKIE").expect("Set ALARM_MFA_COOKIE environment variable");
    alarm.set_trusted_device_cookie(mfa_cookie);

    // Login
    println!("Logging in as {username}...");
    alarm.login().await?;
    println!("Logged in successfully.");

    // Get partition ID
    let partition_id = match partition_id_arg {
        Some(id) => id,
        None => {
            println!("No partition ID specified, fetching first partition...");
            let partitions = alarm.fetch_partitions().await?;
            let first = partitions.first().ok_or("no partitions found")?;
            println!(
                "Using partition: {} ({}): {}",
                first.name, first.id, first.state
            );
            first.id.clone()
        }
    };

    // Execute command
    match command.as_str() {
        "stay" => {
            println!("Arming partition {partition_id} in Stay mode...");
            alarm.arm_stay(&partition_id, &[]).await?;
            println!("Armed Stay.");
        }
        "away" => {
            println!("Arming partition {partition_id} in Away mode...");
            alarm.arm_away(&partition_id, &[]).await?;
            println!("Armed Away.");
        }
        "night" => {
            println!("Arming partition {partition_id} in Night mode...");
            alarm.arm_night(&partition_id, &[]).await?;
            println!("Armed Night.");
        }
        "disarm" => {
            println!("Disarming partition {partition_id}...");
            alarm.disarm(&partition_id).await?;
            println!("Disarmed.");
        }
        "stay-bypass" => {
            println!("Arming partition {partition_id} in Stay mode (bypass open sensors)...");
            alarm.arm_stay(&partition_id, &[ArmingOption::BypassSensors])
                .await?;
            println!("Armed Stay (bypassed).");
        }
        "away-bypass" => {
            println!("Arming partition {partition_id} in Away mode (bypass open sensors)...");
            alarm.arm_away(&partition_id, &[ArmingOption::BypassSensors])
                .await?;
            println!("Armed Away (bypassed).");
        }
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Valid commands: stay, away, night, disarm, stay-bypass, away-bypass");
            std::process::exit(1);
        }
    }

    Ok(())
}
