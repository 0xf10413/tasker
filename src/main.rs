use std::env;
use std::sync::Arc;

mod sql_connection_factory;
mod task;
mod task_repo;
mod webapp;

use tokio::signal;

use crate::sql_connection_factory::SqliteConnectionFactory;
use crate::task_repo::{TaskRepo, TaskRepoError};
use crate::webapp::{AppState, build_app};

const TASKER_PORT_ENV_VAR: &str = "TASKER_PORT";
const TASKER_DEFAULT_PORT: i32 = 3000;

#[allow(dead_code)] // Rust has no way to know where this is used
#[derive(Debug)]
enum ApplicativeError {
    TaskRepoError(TaskRepoError),
    IoError(std::io::Error),
}

impl From<TaskRepoError> for ApplicativeError {
    fn from(value: TaskRepoError) -> Self {
        ApplicativeError::TaskRepoError(value)
    }
}

impl From<std::io::Error> for ApplicativeError {
    fn from(value: std::io::Error) -> Self {
        ApplicativeError::IoError(value)
    }
}

#[tokio::main]
async fn main() -> Result<(), ApplicativeError> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Database setup
    TaskRepo::new(Arc::new(SqliteConnectionFactory {})).init_db()?;

    // Routing setup
    let app_state = AppState {
        connection_factory: Arc::new(SqliteConnectionFactory {}),
    };
    let app = build_app(app_state);

    // Finding port configuration
    let bind_port: i32 = match env::var(TASKER_PORT_ENV_VAR) {
        Ok(val) => match val.to_string().parse::<i32>() {
            Ok(val) => val,
            Err(_) => TASKER_DEFAULT_PORT,
        },
        Err(_) => TASKER_DEFAULT_PORT,
    };
    let bind_ip_port: String = format!("0.0.0.0:{}", bind_port);

    let listener = tokio::net::TcpListener::bind(bind_ip_port).await?;
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            println!("Received ctrl-c signal, stopping...")
        },
        _ = terminate => {
            println!("Received terminate signal, stopping...")
        },
    }
}
