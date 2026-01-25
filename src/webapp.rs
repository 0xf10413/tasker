use std::sync::Arc;

use crate::sql_connection_factory::SqlConnectionFactory;
use crate::task::Task;
use crate::task::TaskError;
use crate::task::TaskId;

use crate::task_repo::{TaskRepo, TaskRepoError};
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::http::StatusCode;
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, IntoResponse, Redirect, Result},
    routing::{get, post},
};
use minijinja::{Environment, context, path_loader};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

impl IntoResponse for TaskRepoError {
    fn into_response(self) -> Response<Body> {
        let body = match self {
            Self::Error { error } => error,
            Self::SqlError { original_error } => original_error.to_string(),
            Self::IoError { original_error } => original_error.to_string(),
            Self::JinjaError { original_error } => original_error.to_string(),
            Self::TaskError { original_error } => original_error.to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl IntoResponse for TaskError {
    fn into_response(self) -> Response<Body> {
        let body = match self {
            Self::PriorityNotInRangeError(c) => format!("Priority {} not in expected range", c),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<minijinja::Error> for TaskRepoError {
    fn from(value: minijinja::Error) -> Self {
        TaskRepoError::JinjaError {
            original_error: value,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub connection_factory: Arc<dyn SqlConnectionFactory>,
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/flag-pending/{task_id}", post(flag_pending))
        .route("/flag-done/{task_id}", post(flag_done))
        .route("/increase-priority/{task_id}", post(increase_priority))
        .route("/lower-priority/{task_id}", post(lower_priority))
        .route("/add-new-task", post(add_new_task))
        .route("/update-description/{task_id}", post(update_description))
        .route("/task-cleanup", post(task_cleanup))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

fn render<S: Serialize>(template: &str, context: S) -> Result<Html<String>, TaskRepoError> {
    let mut minijinja_env = Environment::new();
    minijinja_env.set_loader(path_loader("assets"));
    let template = minijinja_env.get_template(template)?;
    Ok(Html(template.render(context)?))
}

async fn root(State(state): State<AppState>) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);
    let all_tasks = task_repo.get_all_tasks()?;

    render("index.html.j2", context! { tasks => all_tasks })
}

#[derive(Deserialize)]
struct AddNewTaskInput {
    priority: char,
    description: String,
}

async fn add_new_task(
    State(state): State<AppState>,
    Form(task_desc): Form<AddNewTaskInput>,
) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let task = Task::new(task_desc.priority, &task_desc.description)?;
    task_repo.persist_task(&task)?;

    Ok(Redirect::to("/"))
}

async fn flag_done(
    State(state): State<AppState>,
    Path(task_id): Path<TaskId>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let mut task = task_repo.get_task(task_id)?;
    task.completed = true;
    task_repo.persist_task(&task)?;

    render("task_row.html.j2", context! { task => task })
}

async fn flag_pending(
    State(state): State<AppState>,
    Path(task_id): Path<TaskId>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let mut task = task_repo.get_task(task_id)?;
    task.completed = false;
    task_repo.persist_task(&task)?;

    render("task_row.html.j2", context! { task => task })
}

async fn increase_priority(
    State(state): State<AppState>,
    Path(task_id): Path<TaskId>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let mut task = task_repo.get_task(task_id)?;
    task.increase_priority();
    task_repo.persist_task(&task)?;

    render("task_row.html.j2", context! { task => task })
}

async fn lower_priority(
    State(state): State<AppState>,
    Path(task_id): Path<TaskId>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let mut task = task_repo.get_task(task_id)?;
    task.lower_priority();
    task_repo.persist_task(&task)?;

    render("task_row.html.j2", context! { task => task })
}

#[derive(Deserialize)]
struct UpdateDescriptionInput {
    task_description: String,
}

async fn update_description(
    State(state): State<AppState>,
    Path(task_id): Path<TaskId>,
    Form(task_description): Form<UpdateDescriptionInput>,
) -> Result<Response<Body>> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let mut task = task_repo.get_task(task_id)?;
    task.description = String::from(task_description.task_description.trim());
    task_repo.persist_task(&task)?;

    Ok(Response::new(Body::empty()))
}

async fn task_cleanup(State(state): State<AppState>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    task_repo.cleanup()?;

    Ok(Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use crate::sql_connection_factory::tests::TempDirSqliteConnectionFactory;

    use super::*;
    use axum::http::{self, Request, header::LOCATION};
    use http_body_util::BodyExt;
    use tower::Service;

    async fn add_new_task(app: &mut Router, priority: char, description: &str) {
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/add-new-task")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(format!(
                        "priority={priority}&description={description}"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/");
    }

    async fn parse_body(response: Response<Body>) -> String {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(body.to_vec()).unwrap()
    }

    async fn get_main_page_body(app: &mut Router) -> String {
        let response = app
            .call(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        parse_body(response).await
    }

    #[tokio::test]
    async fn full_usage() {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new().unwrap());
        TaskRepo::new(connection_factory.clone()).init_db().unwrap();

        let mut app = build_app(AppState {
            connection_factory: connection_factory,
        });

        // Add new task
        add_new_task(&mut app, 'B', "SomeTask").await;

        // Ensure it appears in the output
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(parsed_body.contains("(B)"));
        assert!(parsed_body.contains("SomeTask"));

        // Increase priority
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/increase-priority/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Ensure priority was increased
        assert!(!parsed_body.contains("(B)"));
        assert!(parsed_body.contains("(A)"));

        // Lower priority
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/lower-priority/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Ensure priority was increased
        assert!(!parsed_body.contains("(A)"));
        assert!(parsed_body.contains("(B)"));

        // Flag as done
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/flag-done/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Ensure task is flagged as done
        assert!(!parsed_body.contains("✓"));
        assert!(parsed_body.contains("✗"));

        // Flag as pending
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/flag-pending/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Ensure task is flagged as pending
        assert!(!parsed_body.contains("✗"));
        assert!(parsed_body.contains("✓"));

        // Update description
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/update-description/1")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from("task_description=SomeNewTask"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Body empty for this request as there is no need for replacement
        assert_eq!(parsed_body.len(), 0);
    }

    #[tokio::test]
    async fn task_cleanup() {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new().unwrap());
        TaskRepo::new(connection_factory.clone()).init_db().unwrap();

        let mut app = build_app(AppState {
            connection_factory: connection_factory,
        });

        // Add new task
        add_new_task(&mut app, 'B', "SomeTask").await;
        add_new_task(&mut app, 'A', "SomeImportantTask").await;
        add_new_task(&mut app, 'C', "SomeNotImportantTask").await;

        // Flag some of them as done
        for i in 1..=2 {
            let response = app
                .call(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri(format!("/flag-done/{i}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // Ensure they are still in the main page
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(parsed_body.contains("SomeTask"));
        assert!(parsed_body.contains("SomeImportantTask"));
        assert!(parsed_body.contains("SomeNotImportantTask"));

        // Trigger cleanup
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/task-cleanup")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/");

        // Ensure they have been deleted
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(!parsed_body.contains("SomeTask")); // Done => removed
        assert!(!parsed_body.contains("SomeImportantTask")); // Done => removed
        assert!(parsed_body.contains("SomeNotImportantTask")); // Pending => kept
    }
}
