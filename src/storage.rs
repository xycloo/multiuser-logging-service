use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tokio_postgres::{types::Type, Error, NoTls, Statement};
use crate::{logs::LogWrapper, IsLog, LogLevel, LoggerStorage};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MercuryLog {
    pub level: LogLevel,
    pub message: String,
    pub data: Option<Vec<u8>>,
}

impl IsLog for MercuryLog {
    fn data(&self) -> Option<Vec<u8>> {
        self.data.clone()
    }

    fn message(&self) -> String {
        self.message.clone()
    }

    fn level(&self) -> LogLevel {
        self.level.clone()
    }
}

impl LoggerStorage {
    pub async fn new(db_path: impl ToString) -> Self {
        let (client, connection) = tokio_postgres::connect(&db_path.to_string(), NoTls)
            .await
            .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Self { client }
    }

    pub async fn db_setup_project(&self) {
        let create_table = String::from(&format!(
            "CREATE TABLE IF NOT EXISTS mercury_user_logs (
                user_id INT8,
                timestamp INT8,
                loglevel INT8,
                message TEXT
            )",
        ));

        let delete_rows = String::from(&format!("DELETE FROM mercury_user_logs",));

        self.client.execute(&create_table, &[]).await.unwrap();
        self.client.execute(&delete_rows, &[]).await.unwrap();
    }

    async fn prepared_statement(&self) -> Result<Statement, Error> {
        self.client.prepare_typed(
            "INSERT INTO mercury_user_logs (user_id, timestamp, loglevel, message) VALUES ($1, $2, $3, $4)",
            &[Type::INT8, Type::INT8, Type::INT8, Type::TEXT],
        ).await
    }

    pub async fn write_log(&self, user_id: i64, log: MercuryLog) -> Result<(), Error> {
        let statement = self.prepared_statement().await?;
        let time = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.client.execute(&statement, &[&user_id, &time, &(log.level as i64), &log.message]).await?;

        Ok(())
    }

    pub async fn write_debug(&self, user_id: i64, timestamp: i64, message: String) -> Result<(), Error> {
        let statement = self.prepared_statement().await?;
        self.client.execute(&statement, &[&user_id, &timestamp, &(LogLevel::Debug as u32), &message]).await?;

        Ok(())
    }

    pub async fn write_warning(&self, user_id: i64, timestamp: i64, message: String) -> Result<(), Error> {
        let statement = self.prepared_statement().await?;
        self.client.execute(&statement, &[&user_id, &timestamp, &(LogLevel::Warning as u32), &message]).await?;

        Ok(())
    }

    pub async fn write_error(&self, user_id: i64, timestamp: i64, message: String) -> Result<(), Error> {
        let statement = self.prepared_statement().await?;
        self.client.execute(&statement, &[&user_id, &timestamp, &(LogLevel::Error as u32), &message]).await?;

        Ok(())
    }

    pub async fn read_user_logs(&self, user_id: i64) -> Result<Vec<LogWrapper<MercuryLog>>, Error> {
        let client = &self.client;
        let query = client
        .prepare_typed(
            "select timestamp, loglevel, message from mercury_user_logs where user_id = $1;",
            &[Type::INT8],
        )
        .await?;

        let query = client.query(&query, &[&user_id]).await?;
        let mut logs = Vec::new();

        for row in query {
            let timestamp: i64 = row.get(0);
            let log_level: i64 = row.get(1);
            let message: String = row.get(2);

            logs.push(LogWrapper {
                time: timestamp,
                inner: MercuryLog {
                    level: LogLevel::from_u32(log_level as u32),
                    message,
                    data: None
                }
            })
        }

        Ok(logs)
    }
}
