mod divera_status1;

use crate::divera_status1::*;

use std::time::Duration;

use clap::{Parser, Subcommand};
use dbus::blocking::Connection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// reload data from api
    Update,
    /// set next status in order
    Next,
    /// set previous status in order
    Prev,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "de.nlih.diverastatus",
        "/de/nlih/DiveraStatus1",
        Duration::from_millis(5000),
    );

    match args.command {
        Commands::Update => proxy.update()?,
        Commands::Next => proxy.next()?,
        Commands::Prev => proxy.previous()?,
    }

    Ok(())
}
