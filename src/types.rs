use std::collections::HashMap;
use std::path::PathBuf;

use clap::ArgGroup;
use clap::Parser;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Parser, Getters)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("token_input").required(true).args(["token", "token_file"])))]
pub struct Arguments {
    /// your personal api token for the divera instance
    #[arg(short, long)]
    pub(crate) token: Option<String>,

    /// file with the api token as first line
    #[arg(short = 'f', long)]
    pub(crate) token_file: Option<PathBuf>,

    /// update interval in seconds
    #[arg(short, long, default_value_t = 30)]
    pub(crate) interval: u8,

    /// divera instance to use
    #[arg(long, default_value = "https://app.divera247.com")]
    pub(crate) server: String,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct ClickEvent {
    button: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Wrapper<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct User {
    #[serde(rename = "stdformat_name")]
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct Status {
    name: String,
    color_hex: String,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct BasicMonitorStatus {
    #[serde(rename = "all")]
    count: u32,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct MonitorUser {
    id: u32,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct MonitorStatus {
    #[serde(rename = "all")]
    users: Vec<MonitorUser>,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct Monitor {
    #[serde(rename = "1")]
    basic: HashMap<String, BasicMonitorStatus>,

    #[serde(rename = "2")]
    complex: HashMap<String, MonitorStatus>,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct UserStatus {
    #[serde(rename(serialize = "id"))]
    status_id: u32,
}

impl UserStatus {
    pub fn new(status_id: u32) -> Self {
        Self { status_id }
    }
}
