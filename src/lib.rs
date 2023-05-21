mod api;
mod api_types;
mod dbus_interface;
mod i3blocks;
mod types;
mod divera_status1;

use crate::api::{Connection, StatusMap, UserMap};
use crate::api_types::{Monitor, UserStatus};
pub use crate::types::Arguments;
use std::collections::HashMap;
use std::str::FromStr;

use std::time::Duration;

use reqwest::Client;
use strfmt::{strfmt, FmtError};

use crate::types::Update;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;

fn format_output(
    args: &Arguments,
    monitor: &Monitor,
    user_status: &UserStatus,
    user_map: &UserMap,
    status_map: &StatusMap,
) -> String {
    let mut data: HashMap<String, String> = HashMap::new();

    // user status
    {
        let user_status = status_map
            .get(&user_status.status_id().to_string())
            .expect("user status not in cached data");
        data.insert("status_name".to_string(), user_status.name().clone());
        data.insert("status_color".to_string(), user_status.color_hex().clone());
    }

    // all statuses
    for (status_id, status) in monitor.complex() {
        let status_color = status_map
            .get(status_id)
            .unwrap_or_else(|| panic!("status {} not in cached data", status_id))
            .color_hex()
            .clone();

        let user_count = monitor.basic().get(status_id).unwrap().count().to_string();

        let user_names: Vec<String> = status
            .users()
            .iter()
            .map(|m_user| {
                user_map
                    .get(&m_user.id().to_string())
                    .unwrap_or_else(|| panic!("user {} not in cached data", m_user.id()))
                    .name()
                    .clone()
            })
            .collect();

        data.insert(status_id.to_string(), user_names.join(", "));
        data.insert(format!("{}_count", status_id), user_count);
        data.insert(format!("{}_color", status_id), status_color);
    }

    // full and short text
    {
        let mut full_statuses: Vec<String> = Vec::new();
        let mut short_statuses: Vec<String> = Vec::new();

        for status in args.shown_statuses.split(',') {
            // check for status with no users
            if data
                .get(&format!("{}_count", status))
                .unwrap_or_else(|| panic!("invalid status in shown_statuses: {}", status))
                == "0"
            {
                continue;
            }

            if args.escape_quotes {
                full_statuses.push(
                    strfmt(
                        &format!("<span color=\\\"#{{{0}_color}}\\\">{{{0}}}</span>", status),
                        &data,
                    )
                        .unwrap(),
                );
                short_statuses.push(
                    strfmt(
                        &format!(
                            "<span color=\\\"#{{{0}_color}}\\\">{{{0}_count}}</span>",
                            status
                        ),
                        &data,
                    )
                        .unwrap(),
                );

            }
            else {
                full_statuses.push(
                    strfmt(
                        &format!("<span color=\"#{{{0}_color}}\">{{{0}}}</span>", status),
                        &data,
                    )
                        .unwrap(),
                );
                short_statuses.push(
                    strfmt(
                        &format!(
                            "<span color=\"#{{{0}_color}}\">{{{0}_count}}</span>",
                            status
                        ),
                        &data,
                    )
                        .unwrap(),
                );

            }
        }

        data.insert("full_text".to_string(), full_statuses.join(" - "));
        data.insert("short_text".to_string(), short_statuses.join("-"));
    }

    match strfmt(&args.display_format, &data) {
        Ok(output) => output,
        Err(err) => match err {
            FmtError::Invalid(err) => {
                panic!("invalid display_format: {}", err)
            }
            FmtError::KeyError(err) => {
                panic!("invalid variable in display_format: {}", err)
            }
            err => {
                panic!("{}", err)
            }
        },
    }
}

async fn wait_update(rx: &mut Receiver<Update>) -> Update {
    rx.recv().await.expect("channel closed by reading thread")
}

pub async fn start(args: Arguments, token: String) -> Result<(), reqwest::Error> {
    // parse args
    let status_order: Vec<u32> = args
        .status_order
        .split(',')
        .map(|status| u32::from_str(status).expect("invalid status id in order"))
        .collect();
    if args.debug {
        println!("debug: using order: {:?}", status_order);
    }

    // set up connection
    let connection = Connection::new(
        Client::builder().https_only(true).build()?,
        args.server().clone(),
        token,
        args.debug,
    );

    // set up event producers
    #[allow(unused)]
    let (tx, mut rx): (Sender<Update>, Receiver<Update>) = mpsc::channel(64);

    #[cfg(feature = "i3blocks")]
    i3blocks::setup(tx.clone(), args.debug);

    #[cfg(feature = "dbus-interface")]
    dbus_interface::setup(tx.clone(), args.debug).await;

    // request initial data
    let (user_map, status_map) = connection.pull_static().await?;
    let (old_monitor, mut old_user_status) = connection.pull_mutable().await?;

    println!(
        "{}",
        format_output(
            &args,
            &old_monitor,
            &old_user_status,
            &user_map,
            &status_map
        )
    );

    if args.debug {
        println!("debug: starting loop");
    }
    loop {
        if let Ok(update) = timeout(
            Duration::from_secs(args.interval as u64),
            wait_update(&mut rx),
        )
        .await
        {
            if args.debug {
                println!("debug: Got event: {:?}", update);
            }
            if update == Update::StatusPrev || update == Update::StatusNext {
                let current_status = status_order
                    .iter()
                    .enumerate()
                    .find(|item| item.1 == old_user_status.status_id());

                let index = current_status.map(|status| status.0).unwrap_or(0) as i32;
                let new_index = if update == Update::StatusPrev {
                    index - 1
                } else {
                    index + 1
                }
                .rem_euclid(status_order.len() as i32);

                connection
                    .set_status_id(status_order[new_index as usize])
                    .await?;
            }
        }

        if args.debug {
            println!("debug: updating");
        }

        let (monitor, user_status) = connection.pull_mutable().await?;

        println!(
            "{}",
            format_output(&args, &monitor, &user_status, &user_map, &status_map)
        );

        old_user_status = user_status;
    }
}

#[cfg(test)]
mod test {
    use crate::api::{StatusMap, UserMap};
    use crate::api_types::{
        BasicMonitorStatus, Monitor, MonitorStatus, MonitorUser, Status, User, UserStatus,
    };
    use crate::{format_output, Arguments};
    use std::collections::HashMap;

    fn get_example_data() -> (Monitor, UserStatus, UserMap, StatusMap) {
        let mut monitor = Monitor {
            basic: HashMap::new(),
            complex: HashMap::new(),
        };
        let user_status = UserStatus { status_id: 2 };
        let mut user_map: UserMap = HashMap::new();
        let mut status_map: StatusMap = HashMap::new();

        user_map.insert(
            "6".to_string(),
            User {
                name: "A".to_string(),
            },
        );
        user_map.insert(
            "7".to_string(),
            User {
                name: "B".to_string(),
            },
        );
        user_map.insert(
            "8".to_string(),
            User {
                name: "C".to_string(),
            },
        );
        user_map.insert(
            "9".to_string(),
            User {
                name: "D".to_string(),
            },
        );

        status_map.insert(
            "1".to_string(),
            Status {
                name: "one".to_string(),
                color_hex: "f1f".to_string(),
            },
        );
        status_map.insert(
            "2".to_string(),
            Status {
                name: "two".to_string(),
                color_hex: "f2f".to_string(),
            },
        );
        status_map.insert(
            "3".to_string(),
            Status {
                name: "three".to_string(),
                color_hex: "f3f".to_string(),
            },
        );
        status_map.insert(
            "4".to_string(),
            Status {
                name: "four".to_string(),
                color_hex: "f4f".to_string(),
            },
        );

        monitor
            .basic
            .insert("1".to_string(), BasicMonitorStatus { count: 2 });
        monitor
            .basic
            .insert("2".to_string(), BasicMonitorStatus { count: 1 });
        monitor
            .basic
            .insert("3".to_string(), BasicMonitorStatus { count: 0 });
        monitor
            .basic
            .insert("4".to_string(), BasicMonitorStatus { count: 1 });

        monitor.complex.insert(
            "1".to_string(),
            MonitorStatus {
                users: vec![MonitorUser { id: 9 }, MonitorUser { id: 6 }],
            },
        );
        monitor.complex.insert(
            "2".to_string(),
            MonitorStatus {
                users: vec![MonitorUser { id: 7 }],
            },
        );
        monitor
            .complex
            .insert("3".to_string(), MonitorStatus { users: vec![] });
        monitor.complex.insert(
            "4".to_string(),
            MonitorStatus {
                users: vec![MonitorUser { id: 8 }],
            },
        );

        (monitor, user_status, user_map, status_map)
    }

    #[test]
    fn test_i3blocks_format() {
        let args = Arguments {
            token: None,
            token_file: None,
            interval: 0,
            server: "".to_string(),
            shown_statuses: "1,3,2".to_string(),
            status_order: "4,1,2,3".to_string(),
            display_format: "{{\"full_text\": \"{full_text} <span color=\"#{status_color}\">@</span>\", \"short_text\": \"{short_text}\"}}".to_string(),
            escape_quotes: true,
            no_pango: false,
            debug: false,
        };
        let (monitor, user_status, user_map, status_map) = get_example_data();

        let output = format_output(&args, &monitor, &user_status, &user_map, &status_map);

        let expected = "{\"full_text\": \"<span color=\"#f1f\">D, A</span> - <span color=\"#f2f\">B</span> <span color=\"#f2f\">@</span>\", \"short_text\": \"<span color=\"#f1f\">2</span>-<span color=\"#f2f\">1</span>\"}".to_string();

        assert_eq!(output, expected);
    }
}
