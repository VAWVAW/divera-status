#![cfg(feature = "dbus-interface")]

use crate::types::Update;
use crate::divera_status1::{DeNlihDiveraStatus1Monitor, DeNlihDiveraStatus1Status, register_de_nlih_divera_status1_monitor, register_de_nlih_divera_status1_status};

use std::sync::Arc;

use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::MethodErr;
use dbus::nonblock::SyncConnection;
use dbus_crossroads::{Crossroads};
use dbus_tokio::connection;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

struct DbusData {
    tx: mpsc::Sender<Update>,
}

impl DbusData {
    fn send_update(&self, update: Update) -> Result<(), MethodErr> {
        self.tx.try_send(update).map_err(|err| match err {
            TrySendError::Full(_) => MethodErr::failed("too many requests"),
            TrySendError::Closed(_) => panic!("update channel was closed"),
        })
    }
}

impl DeNlihDiveraStatus1Monitor for DbusData {
    fn update(&mut self) -> Result<(), MethodErr> {
        self.send_update(Update::Reload)
    }
}

impl DeNlihDiveraStatus1Status for DbusData {
    fn next(&mut self) -> Result<(), MethodErr> {
        self.send_update(Update::StatusNext)
    }

    fn previous(&mut self) -> Result<(), MethodErr> {
        self.send_update(Update::StatusPrev)
    }
}

pub async fn setup(tx: mpsc::Sender<Update>, debug: bool) {
    if debug {
        println!("debug: setting up dbus connection")
    }

    // set up async dbus connection
    let (resource, con): (connection::IOResource<SyncConnection>, Arc<SyncConnection>) =
        match connection::new_session_sync() {
            Ok(data) => data,
            Err(error) => panic!("error connecting to session dbus: {}", error),
        };
    let _handle = tokio::spawn(async {
        let err = resource.await;
        panic!("lost connection to dbus: {}", err)
    });
    let _ = con
        .request_name("de.nlih.diverastatus", false, true, false)
        .await
        .map_err(|err| panic!("error requesting dbus name: {}", err));

    // set up async crossroads
    let mut cr = Crossroads::new();
    cr.set_async_support(Some((
        con.clone(),
        Box::new(|x| {
            tokio::spawn(x);
        }),
    )));

    let dbus_data = DbusData { tx };
    let status_token = register_de_nlih_divera_status1_status(&mut cr);
    let monitor_token = register_de_nlih_divera_status1_monitor(&mut cr);
    cr.insert(
        "/de/nlih/DiveraStatus1",
        &[status_token, monitor_token],
        dbus_data,
    );

    con.start_receive(
        MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );
}
