use crate::types::{Monitor, Status, User, UserStatus, Wrapper};

use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

pub type UserMap = HashMap<String, User>;
pub type StatusMap = HashMap<String, Status>;

#[derive(Debug)]
pub struct Connection {
    client: Client,
    server: String,
    token: String,
}

impl Connection {
    pub fn new(client: Client, server: String, token: String) -> Self {
        Connection {
            client,
            server,
            token,
        }
    }

    async fn make_get_request<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        query: &T,
    ) -> Result<Response, Box<dyn Error>> {
        let response = self
            .client
            .get(self.server.clone() + endpoint)
            .query(query)
            .query(&[("accesskey", self.token.clone())])
            .send()
            .await?;

        response.error_for_status_ref()?;

        Ok(response)
    }

    async fn make_post_request<T: Into<reqwest::Body>>(
        &self,
        endpoint: &str,
        data: T,
    ) -> Result<Response, Box<dyn Error>> {
        let response = self
            .client
            .post(self.server.clone() + endpoint)
            .query(&[("accesskey", self.token.clone())])
            .header("Content-Type", "application/json")
            .body(data)
            .send()
            .await?;

        response.error_for_status_ref()?;

        Ok(response)
    }

    pub async fn pull_static(&self) -> Result<(UserMap, StatusMap), Box<dyn Error>> {
        #[derive(Debug, Deserialize)]
        struct Cluster {
            consumer: HashMap<String, User>,
            status: HashMap<String, Status>,
        }

        #[derive(Debug, Deserialize)]
        struct PullData {
            cluster: Cluster,
        }

        let time_now = format!(
            "{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        let query = [
            ("ts_user", time_now.as_str()),
            ("ts_alarm", time_now.as_str()),
            ("ts_news", time_now.as_str()),
            ("ts_event", time_now.as_str()),
            ("ts_status", time_now.as_str()),
            ("ts_statusplan", time_now.as_str()),
            ("ts_localmonitor", time_now.as_str()),
            ("ts_monitor", time_now.as_str()),
        ];
        let response = self.make_get_request("/api/v2/pull/all", &query).await?;
        let pull_data: Wrapper<PullData> = response.json().await?;

        Ok((
            pull_data.data.cluster.consumer,
            pull_data.data.cluster.status,
        ))
    }

    pub async fn pull_mutable(&self) -> Result<(Monitor, UserStatus), Box<dyn Error>> {
        #[derive(Debug, Deserialize)]
        struct PullData {
            monitor: Monitor,
            status: UserStatus,
        }

        let time_now = format!(
            "{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );

        let query = [
            ("ts_user", time_now.as_str()),
            ("ts_alarm", time_now.as_str()),
            ("ts_news", time_now.as_str()),
            ("ts_event", time_now.as_str()),
            ("ts_statusplan", time_now.as_str()),
            ("ts_localmonitor", time_now.as_str()),
            ("ts_cluster", time_now.as_str()),
        ];
        let response = self.make_get_request("/api/v2/pull/all", &query).await?;
        let pull_data: Wrapper<PullData> = response.json().await?;

        Ok((pull_data.data.monitor, pull_data.data.status))
    }

    pub async fn set_status(&self, data: UserStatus) -> Result<(), Box<dyn Error>> {
        #[derive(Serialize)]
        struct Wrapper {
            #[serde(rename = "Status")]
            status: UserStatus,
        }

        let wrapper = Wrapper { status: data };

        let body = serde_json::to_string(&wrapper)?;
        self.make_post_request("/api/v2/statusgeber/set-status", body)
            .await?;
        Ok(())
    }
}
