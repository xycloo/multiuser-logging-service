use std::time::SystemTime;

use serde::{Deserialize, Serialize};

pub trait IsLog: Clone {
    fn message(&self) -> String;
    fn data(&self) -> Option<Vec<u8>>;
    fn level(&self) -> LogLevel;
}
/* 
/// Permitted log levels.
/// More may be added in the future.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum LogLevel {
    Error,
    Warning,
    Debug,
}*/

#[derive(Clone, Serialize, Deserialize, Debug)]
#[repr(u32)]
pub enum LogLevel {
    Debug = 0,
    Warning = 1,
    Error = 2,
}

impl LogLevel {
    pub fn from_u32(n: u32) -> Self {
        match n {
            0 => Self::Debug,
            1 => Self::Warning,
            _ => Self::Error,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogWrapper<L> {
    pub time: i64,
    pub inner: L,
}

impl<L: IsLog> LogWrapper<L> {
    pub(crate) fn inner(&self) -> &L {
        &self.inner
    }

    pub(crate) fn time(&self) -> i64 {
        self.time
    }
}

// Note: UserLogsGroup container has already been locked at this point.
#[derive(Clone)]
pub struct UserLogsGroup<L> {
    is_logging: bool,
    error: Vec<LogWrapper<L>>,
    warn: Vec<LogWrapper<L>>,
    debug: Vec<LogWrapper<L>>,
}

impl<L: IsLog> UserLogsGroup<L> {
    pub fn new() -> Self {
        Self {
            is_logging: false,
            error: vec![],
            warn: vec![],
            debug: vec![],
        }
    }

    pub(crate) fn clear(&mut self) {
        // TODO: add option to archive logs in storage if needed.

        self.error.clear();
        self.warn.clear();
        self.debug.clear()
    }

    pub(crate) fn add_error(&mut self, log: L) {
        if !self.is_logging {
            return;
        }

        let time = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let log = LogWrapper { time, inner: log };

        self.error.push(log)
    }

    pub(crate) fn add_warning(&mut self, log: L) {
        if !self.is_logging {
            return;
        }

        let time = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let log = LogWrapper { time, inner: log };

        self.warn.push(log)
    }

    pub(crate) fn add_debug(&mut self, log: L) {
        if !self.is_logging {
            return;
        }

        let time = std::time::SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let log = LogWrapper { time, inner: log };

        self.debug.push(log)
    }

    pub fn is_logging(&mut self) {
        self.is_logging = true
    }

    pub fn is_not_logging(&mut self) {
        self.is_logging = false
    }

    pub fn errors(&self) -> &Vec<LogWrapper<L>> {
        &self.error
    }

    pub fn debug(&self) -> &Vec<LogWrapper<L>> {
        &self.debug
    }

    pub fn warning(&self) -> &Vec<LogWrapper<L>> {
        &self.warn
    }
}
