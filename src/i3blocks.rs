#![cfg(feature = "i3blocks")]
use crate::types::Update;

use std::io::stdin;
use std::thread;

use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Debug, Deserialize)]
struct ClickEvent {
    button: u32,
}

fn read_stdin(tx: mpsc::Sender<Update>, debug: bool) {
    let stdin = stdin();
    if debug {
        println!("debug: reading on stdin");
    }
    loop {
        let mut buffer = String::new();
        let _ = stdin.read_line(&mut buffer);
        if debug {
            println!("debug: got input on stdin: '{}'", buffer);
        }
        let event: ClickEvent =
            serde_json::from_str(buffer.as_str()).expect("got invalid json as click event");

        let to_send = match event.button {
            // mousewheel up
            4 => Update::StatusNext,
            // mousewheel down
            5 => Update::StatusPrev,
            _ => Update::Reload,
        };
        tx.blocking_send(to_send)
            .expect("channel closed by main thread");
    }
}

pub fn setup(tx: mpsc::Sender<Update>, debug: bool) {
    thread::spawn(move || read_stdin(tx, debug));
}
