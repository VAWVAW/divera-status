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
    pub(crate) name: String,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct Status {
    pub(crate) name: String,
    pub(crate) color_hex: String,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct BasicMonitorStatus {
    #[serde(rename = "all")]
    pub(crate) count: u32,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct MonitorUser {
    pub(crate) id: u32,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct MonitorStatus {
    #[serde(rename = "all")]
    pub(crate) users: Vec<MonitorUser>,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct Monitor {
    #[serde(rename = "1")]
    pub(crate) basic: HashMap<String, BasicMonitorStatus>,

    #[serde(rename = "2")]
    pub(crate) complex: HashMap<String, MonitorStatus>,
}

#[derive(Debug, Serialize, Deserialize, Getters, Clone)]
pub struct UserStatus {
    #[serde(rename(serialize = "id"))]
    pub(crate) status_id: u32,
}

impl UserStatus {
    pub fn new(status_id: u32) -> Self {
        Self { status_id }
    }
}
