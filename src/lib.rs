use std::{collections::HashMap, sync::Arc};
use logs::{LogWrapper, UserLogsGroup};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
pub use logs::{IsLog, LogLevel};

mod logs;

#[derive(Clone)]
pub struct Logger<L> {
    state: Arc<Mutex<HashMap<i64, UserLogsGroup<L>>>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServiceLog {
    level: LogLevel,
    message: String,
    data: Option<Vec<u8>>,
    time: i64
}

impl<L: IsLog> From<&LogWrapper<L>> for ServiceLog {
    fn from(value: &LogWrapper<L>) -> Self {
        Self {
            level: value.inner().level(),
            message: value.inner().message(),
            data: value.inner().data(),
            time: value.time()
        }
    }
}

impl<L: IsLog> Logger<L> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    /// Unified view of all the logs
    pub async fn read_log(&self, user_id: i64) -> Vec<ServiceLog> {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            let mut all_logs_casted = Vec::new();

            for log in user_logs.errors() {
                all_logs_casted.push(log.into())
            }

            for log in user_logs.debug() {
                all_logs_casted.push(log.into())
            }

            for log in user_logs.warning() {
                all_logs_casted.push(log.into())
            }

            all_logs_casted
        } else {
            vec![]
        }
    }

    pub async fn is_logging(&self, user_id: i64) {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            user_logs.is_logging()
        } else {
            return;
        }
    }

    pub async fn is_not_logging(&self, user_id: i64) {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            user_logs.is_not_logging()
        } else {
            return;
        }
    }

    pub async fn write_log(&self, user_id: i64, log: L) {
        match log.level() {
            LogLevel::Error => self.write_error(user_id, log).await,
            LogLevel::Debug => self.write_debug(user_id, log).await,
            LogLevel::Warning => self.write_warning(user_id, log).await
        }
    }

    pub async fn write_error(&self, user_id: i64, log: L) {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            user_logs.add_error(log);
        } else {
            let mut logs = UserLogsGroup::new();
            logs.add_error(log);
            state.insert(user_id, logs);
        }
    }

    pub async fn write_warning(&self, user_id: i64, log: L) {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            user_logs.add_warning(log)
        } else {
            let mut logs = UserLogsGroup::new();
            logs.add_warning(log);
            state.insert(user_id, logs);
        }
    }

    pub async fn write_debug(&self, user_id: i64, log: L) {
        let mut state = self.state.lock().await;
        
        let user_logs = state.get_mut(&user_id);

        if let Some(user_logs) = user_logs {
            user_logs.add_debug(log)
        } else {
            let mut logs = UserLogsGroup::new();
            logs.add_debug(log);
            state.insert(user_id, logs);
        }
    }

    pub async fn read_errros(&self, user_id: i64) -> Vec<LogWrapper<L>> {
        let state = self.state.lock().await;
        
        let user_logs = state.get(&user_id);
        if user_logs.is_none() {
            return vec![];
        }

        let user_logs = user_logs.unwrap();
        user_logs.errors().clone()
    }

    pub async fn read_debug(&self, user_id: i64) -> Vec<LogWrapper<L>> {
        let state = self.state.lock().await;
        
        let user_logs = state.get(&user_id);
        if user_logs.is_none() {
            return vec![];
        }

        let user_logs = user_logs.unwrap();
        user_logs.debug().clone()
    }

    pub async fn read_warning(&self, user_id: i64) -> Vec<LogWrapper<L>> {
        let state = self.state.lock().await;
        
        let user_logs = state.get(&user_id);
        if user_logs.is_none() {
            return vec![];
        }

        let user_logs = user_logs.unwrap();
        user_logs.warning().clone()
    }
}

