use crate::api_types::{Monitor, Status, User, UserStatus, Wrapper};

use std::collections::HashMap;
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
    debug: bool,
}

impl Connection {
    pub fn new(client: Client, server: String, token: String, debug: bool) -> Self {
        Connection {
            client,
            server,
            token,
            debug,
        }
    }

    async fn make_get_request<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        query: &T,
    ) -> Result<Response, reqwest::Error> {
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
    ) -> Result<Response, reqwest::Error> {
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

    pub async fn pull_static(&self) -> Result<(UserMap, StatusMap), reqwest::Error> {
        #[derive(Debug, Deserialize)]
        struct Cluster {
            consumer: HashMap<String, User>,
            status: HashMap<String, Status>,
        }

        #[derive(Debug, Deserialize)]
        struct PullData {
            cluster: Cluster,
        }

        if self.debug {
            println!("debug: pulling static data");
        }

        let time_now = format!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before 1970")
                .as_secs()
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

        if self.debug {
            println!("debug: got pull data: {:?}", pull_data);
        }

        Ok((
            pull_data.data.cluster.consumer,
            pull_data.data.cluster.status,
        ))
    }

    pub async fn pull_mutable(&self) -> Result<(Monitor, UserStatus), reqwest::Error> {
        #[derive(Debug, Deserialize)]
        struct PullData {
            monitor: Monitor,
            status: UserStatus,
        }

        if self.debug {
            println!("debug: pulling mutable data");
        }

        let time_now = format!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time is before 1970")
                .as_secs()
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

        if self.debug {
            println!("debug: got pull data: {:?}", pull_data);
        }

        Ok((pull_data.data.monitor, pull_data.data.status))
    }

    pub async fn set_status(&self, data: UserStatus) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        struct Wrapper {
            #[serde(rename = "Status")]
            status: UserStatus,
        }

        if self.debug {
            println!("debug: setting status to: {}", data.status_id());
        }

        let wrapper = Wrapper { status: data };

        let body = serde_json::to_string(&wrapper).unwrap();
        self.make_post_request("/api/v2/statusgeber/set-status", body)
            .await?;
        Ok(())
    }

    pub async fn set_status_id(&self, id: u32) -> Result<(), reqwest::Error> {
        self.set_status(UserStatus::new(id)).await
    }
}
