use std::sync::Arc;

use multiuser_logging_service::{IsLog, LogLevel, Logger};
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::WithStatus, Filter};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct LogClientRequest {
    serialized: Vec<u8>
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZephyrLog {
    pub level: LogLevel,
    pub message: String,
    pub data: Option<Vec<u8>>,
}

impl IsLog for ZephyrLog {
    fn data(&self) -> Option<Vec<u8>> {
        self.data.clone()
    }
    
    fn message(&self) -> String {
        self.message.clone()
    }

    fn level(&self) -> LogLevel {
        self.level
    }
}

fn with_db(db: Arc<Logger<ZephyrLog>>) -> impl Filter<Extract = (Arc<Logger<ZephyrLog>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

#[tokio::main]
async fn main() {
    let logger: Logger<ZephyrLog> = Logger::new();
    let arc = Arc::new(logger);

    let get_errors = warp::path!("error" / i64).and(warp::get()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            let errors = state.read_errros(user_id).await;
    
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(serde_json::to_string(&errors).unwrap(), warp::http::StatusCode::OK))
        }
    }
    );

    let get_warning = warp::path!("warning" / i64).and(warp::get()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            let errors = state.read_warning(user_id).await;
    
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(serde_json::to_string(&errors).unwrap(), warp::http::StatusCode::OK))
        }
    }
    );

    let get_debug = warp::path!("debug" / i64).and(warp::get()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            let errors = state.read_debug(user_id).await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(serde_json::to_string(&errors).unwrap(), warp::http::StatusCode::OK))
        }
    }
    );

    let add_log = warp::path!("log" / i64).and(warp::post()).and(warp::body::json()).and(with_db(arc.clone())).and_then(move |user_id, log: LogClientRequest, state: Arc<Logger<ZephyrLog>>| {
        async move {
            let deserialized = bincode::deserialize(&log.serialized).unwrap();    
            
            println!("Adding log {:?}", deserialized);
            
            state.write_log(user_id, deserialized).await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status("success".into(), warp::http::StatusCode::CREATED))
        }
    }
    );

    let get_logs = warp::path!("log" / i64).and(warp::get()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            let logs = state.read_log(user_id).await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(serde_json::to_string(&logs).unwrap(), warp::http::StatusCode::OK))
        }
    }
    );

    let is_logging = warp::path!("logging" / i64).and(warp::post()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            state.is_logging(user_id).await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status("success".into(), warp::http::StatusCode::OK))
        }
    }
    );

    let is_not_logging = warp::path!("not_logging" / i64).and(warp::post()).and(with_db(arc.clone())).and_then(move |user_id, state: Arc<Logger<ZephyrLog>>| {
        async move {
            state.is_not_logging(user_id).await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status("success".into(), warp::http::StatusCode::OK))
        }
    }
    );

    let get_users = warp::path!("users").and(warp::get()).and(with_db(arc.clone())).and_then(move |state: Arc<Logger<ZephyrLog>>| {
        async move {
            let users = state.read_users().await;
            
            Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(serde_json::to_string(&users).unwrap(), warp::http::StatusCode::OK))
        }
    }
    );


    let routes = warp::post().and(add_log).or(get_debug).or(get_warning).or(get_errors).or(get_logs).or(is_logging).or(is_not_logging).or(get_users);
    warp::serve(routes).run(([0,0,0,0], 8082)).await;
}

#[cfg(test)]
mod test_utils {
    use crate::ZephyrLog;

    #[test]
    fn sample_error_log_serialize() {
        let log = ZephyrLog {
            level: crate::LogLevel::Error,
            message: "test".into(),
            data: None
        };

        println!("{:?}", bincode::serialize(&log).unwrap())
    }

    #[test]
    fn sample_debug_log_serialize() {
        let log = ZephyrLog {
            level: crate::LogLevel::Debug,
            message: "test".into(),
            data: None
        };

        println!("{:?}", bincode::serialize(&log).unwrap())
    }

    #[test]
    fn sample_warning_log_serialize() {
        let log = ZephyrLog {
            level: crate::LogLevel::Warning,
            message: "test".into(),
            data: None
        };

        println!("{:?}", bincode::serialize(&log).unwrap())
    }
}