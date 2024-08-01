//! Simple structs to send logs to the service.

use reqwest::Client;

use crate::{logs::LogWrapper, LogLevel, MercuryLog};

pub struct LoggingClient {
    client: Client
}

impl LoggingClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new()
        }
    }

    pub async fn send_log(&self, user_id: i64, log_level: LogLevel, message: String) -> Result<reqwest::Response, reqwest::Error> {
        let log = MercuryLog {
            level: log_level,
            message,
            data: None
        };

        self.client.post(format!("http://127.0.0.1:8088/logs/{}", user_id)).json(&log).send().await
    }

    pub async fn read_log(&self, user_id: i64) -> Result<Vec<LogWrapper<MercuryLog>>, reqwest::Error> {
        let resp = reqwest::get(format!("http://127.0.0.1:8088/logs/{}", user_id)).await?;
        let resp: Vec<LogWrapper<MercuryLog>> = resp.json().await?;

        Ok(resp)
    }
}
