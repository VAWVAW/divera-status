use std::collections::HashMap;

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

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
