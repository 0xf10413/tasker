use std::env;

mod task;
mod task_repo;
mod webapp;

use crate::task_repo::TaskRepo;
use crate::webapp::build_app;

const TASKER_PORT_ENV_VAR: &str = "TASKER_PORT";
const TASKER_DEFAULT_PORT: i32 = 3000;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Database setup
    TaskRepo::new(None).init_db();

    // Routing setup
    let app = build_app();

    // Finding port configuration
    let bind_port: i32 = match env::var(TASKER_PORT_ENV_VAR) {
        Ok(val) => match val.to_string().parse::<i32>() {
            Ok(val) => val,
            Err(_) => TASKER_DEFAULT_PORT,
        },
        Err(_) => TASKER_DEFAULT_PORT,
    };
    let bind_ip_port: String = format!("0.0.0.0:{}", bind_port);

    // TODO: remove `unwrap` here
    let listener = tokio::net::TcpListener::bind(bind_ip_port).await.unwrap();
    let _ = axum::serve(listener, app).await;
}
