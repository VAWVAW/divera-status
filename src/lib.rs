mod api;
mod types;

pub use crate::types::Arguments;

use crate::api::{Connection, StatusMap, UserMap};
use crate::types::{ClickEvent, Monitor, UserStatus};

use std::fs::File;
use std::io::{stdin, BufRead};
use std::time::Duration;
use std::{io, thread};

use reqwest::Client;

use tokio::sync::mpsc;
use tokio::time::timeout;

pub const SHOWN_STATUSES: [&str; 3] = ["800", "801", "802"];

fn i3blocks_format_monitor(
    monitor: &Monitor,
    user_status: &UserStatus,
    user_map: &UserMap,
    status_map: &StatusMap,
) -> String {
    let mut full_text: Vec<String> = Vec::new();
    let mut short_text: Vec<String> = Vec::new();

    for status in SHOWN_STATUSES {
        let user_count = *monitor
            .basic()
            .get(status)
            .expect("status not found in monitor data")
            .count();
        if user_count == 0 {
            continue;
        }

        let color = status_map
            .get(status)
            .expect("status not in pulled data")
            .color_hex();
        let users: Vec<&str> = monitor
            .complex()
            .get(status)
            .expect("status not found in monitor")
            .users()
            .iter()
            .map(|m_user| {
                user_map
                    .get(&m_user.id().to_string())
                    .expect("user from monitor not in pulled data (maybe restart)")
                    .name()
                    .as_str()
            })
            .collect();

        full_text.push(format!(
            "<span color=\\\"#{}\\\">{}</span>",
            &color,
            users.join(", ")
        ));
        short_text.push(format!(
            "<span color=\\\"#{}\\\">{}</span>",
            &color, user_count
        ));
    }

    let user_color = status_map
        .get(&user_status.status_id().to_string())
        .expect("user status not in pulled data")
        .color_hex();
    let status_string = format!("<span color=\\\"#{}\\\">◼</span>", user_color);

    format!(
        "{{\"full_text\":\"{} {}\", \"short_text\":\"{} {}\"}}",
        full_text.join(" - "),
        &status_string,
        short_text.join("-"),
        &status_string
    )
}

fn read_stdin(tx: mpsc::Sender<ClickEvent>) {
    let stdin = stdin();
    loop {
        let mut buffer = String::new();
        let _ = stdin.read_line(&mut buffer);
        let event: ClickEvent =
            serde_json::from_str(buffer.as_str()).expect("got invalid json as click event");
        tx.blocking_send(event)
            .expect("channel closed by main thread");
    }
}
async fn process_stdin(rx: &mut mpsc::Receiver<ClickEvent>) -> ClickEvent {
    rx.recv().await.expect("channel closed by reading thread")
}

pub fn get_token(args: &Arguments) -> Result<String, io::Error> {
    let token: String = if let Some(token) = &args.token {
        token.clone()
    } else {
        let mut buffer = String::new();
        let file = File::open(
            args.token_file
                .as_ref()
                .expect("neither token or token-file provided"),
        )?;
        let _ = io::BufReader::new(file).read_line(&mut buffer)?;
        buffer.pop();
        buffer
    };
    Ok(token)
}

pub async fn run(args: Arguments, token: String) -> Result<(), reqwest::Error> {
    let connection = Connection::new(
        Client::builder().https_only(true).build()?,
        args.server().clone(),
        token,
    );

    let (tx, mut rx) = mpsc::channel(64);
    thread::spawn(move || read_stdin(tx));

    let (user_map, status_map) = connection.pull_static().await?;
    let (old_monitor, mut old_user_status) = connection.pull_mutable().await?;

    println!(
        "{}",
        i3blocks_format_monitor(&old_monitor, &old_user_status, &user_map, &status_map)
    );

    loop {
        if let Ok(input) = timeout(
            Duration::from_secs(args.interval as u64),
            process_stdin(&mut rx),
        )
        .await
        {
            // button 4 is wheel up and 5 is down
            // possible status ids are 80* or over 20000
            match old_user_status.status_id() + 10 * input.button() {
                // 800: "Auf der Wache"
                840 => {
                    connection.set_status(UserStatus::new(804)).await?;
                }
                850 => {
                    connection.set_status(UserStatus::new(804)).await?;
                }
                // 801: "Auf Anfahrt"
                841 => {
                    connection.set_status(UserStatus::new(800)).await?;
                }
                851 => {
                    connection.set_status(UserStatus::new(804)).await?;
                }
                // 802: "Verfügbar zum Alarm"
                842 => {
                    connection.set_status(UserStatus::new(801)).await?;
                }
                852 => {
                    connection.set_status(UserStatus::new(804)).await?;
                }
                // 804: "Abwesend"
                844 => {
                    connection.set_status(UserStatus::new(801)).await?;
                }
                854 => {
                    connection.set_status(UserStatus::new(802)).await?;
                }
                _ => {}
            }
        }

        let (monitor, user_status) = connection.pull_mutable().await?;

        println!(
            "{}",
            i3blocks_format_monitor(&monitor, &user_status, &user_map, &status_map)
        );

        old_user_status = user_status;
    }
}
