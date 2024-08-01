use multiuser_logging_service::{LoggerStorage, MercuryLog};
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::WithStatus, Filter};


#[tokio::main]
async fn main() {
    let logs = LoggerStorage::new(std::env::var("DB").unwrap()).await;
    logs.db_setup_project().await;

    let add_log = warp::path!("logs" / i64)
        .and(warp::post())
        .and(warp::body::json())
        .and_then(
            move |user_id, log: MercuryLog| async move {
                let logs = LoggerStorage::new(std::env::var("DB").unwrap()).await;
                logs.write_log(user_id, log).await.unwrap();

                Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(
                    "success".into(),
                    warp::http::StatusCode::CREATED,
                ))
            },
        );

    let get_logs = warp::path!("logs" / i64)
        .and(warp::get())
        .and_then(
            move |user_id| async move {
                let logs = LoggerStorage::new(std::env::var("DB").unwrap()).await;
                let logs = logs.read_user_logs(user_id).await.unwrap();
                
                Ok::<WithStatus<String>, Rejection>(warp::reply::with_status(
                    serde_json::to_string(&logs).unwrap(),
                    warp::http::StatusCode::OK,
                ))
            },
        );

    let routes = warp::post()
        .and(add_log)
        .or(get_logs);
    warp::serve(routes).run(([0, 0, 0, 0], 8088)).await;
}


#[test]
fn t() {
    println!("{}", serde_json::to_string(&MercuryLog {
        level: multiuser_logging_service::LogLevel::Error,
        message: "Test log".try_into().unwrap(),
        data: None
    }).unwrap())
}