mod api;
mod api_types;
mod i3blocks;
mod types;

use crate::api::{Connection, StatusMap, UserMap};
use crate::api_types::{Monitor, UserStatus};
pub use crate::types::Arguments;

use std::time::Duration;

use reqwest::Client;

use crate::types::Update;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;

pub const SHOWN_STATUSES: [&str; 3] = ["800", "801", "802"];

// todo update
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

async fn wait_update(rx: &mut Receiver<Update>) -> Update {
    rx.recv().await.expect("channel closed by reading thread")
}

pub async fn run(args: Arguments, token: String) -> Result<(), reqwest::Error> {
    // todo move to Arguments
    // Abwesend, Verfügbar, Auf Anfahrt, Auf Wache
    let status_order = vec![804, 802, 801, 800];

    // setup connection
    let connection = Connection::new(
        Client::builder().https_only(true).build()?,
        args.server().clone(),
        token,
        args.debug,
    );

    // setup event producers
    #[allow(unused)]
    let (tx, mut rx): (Sender<Update>, Receiver<Update>) = mpsc::channel(64);

    #[cfg(feature = "i3blocks")]
    i3blocks::setup(tx.clone(), args.debug);

    // request initial data
    let (user_map, status_map) = connection.pull_static().await?;
    let (old_monitor, mut old_user_status) = connection.pull_mutable().await?;

    println!(
        "{}",
        i3blocks_format_monitor(&old_monitor, &old_user_status, &user_map, &status_map)
    );

    if args.debug {
        println!("debug: starting loop");
    }
    loop {
        if let Ok(update) = timeout(
            Duration::from_secs(args.interval as u64),
            wait_update(&mut rx),
        ).await {
            if args.debug {
                println!("debug: Got event: {:?}", update);
            }
            if update == Update::StatusPrev || update == Update::StatusNext {
                let current_status = status_order.iter().enumerate().find(|item| item.1 == old_user_status.status_id());

                let index = current_status.map(|status| status.0).unwrap_or(0) as i32;
                let new_index = if update == Update::StatusPrev {
                    index - 1
                }
                else {
                    index + 1
                }.rem_euclid(status_order.len() as i32);

                connection.set_status_id(status_order[new_index as usize]).await?;
            }
        }

        if args.debug {
            println!("debug: updating");
        }

        let (monitor, user_status) = connection.pull_mutable().await?;

        println!(
            "{}",
            i3blocks_format_monitor(&monitor, &user_status, &user_map, &status_map)
        );

        old_user_status = user_status;
    }
}
