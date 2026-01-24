use crate::task::Task;
use crate::task::TaskError;
use crate::task::TaskId;

use crate::task_repo::{TaskRepo, TaskRepoError};
use axum::body::Body;
use axum::http::StatusCode;
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, IntoResponse, Redirect, Response, Result},
    routing::{get, post},
};
use minijinja::{Environment, context, path_loader};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

impl IntoResponse for TaskRepoError {
    fn into_response(self) -> Response<Body> {
        let body = match self {
            Self::Error { error } => error,
            Self::SqlError { original_error } => original_error.to_string(),
            Self::JinjaError { original_error } => original_error.to_string(),
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

pub fn build_app() -> Router {
    return Router::new()
        .route("/", get(root))
        .route("/set-pending/{task_id}", post(set_pending))
        .route("/set-done/{task_id}", post(set_done))
        .route("/increase-priority/{task_id}", post(increase_priority))
        .route("/lower-priority/{task_id}", post(lower_priority))
        .route("/add-new-task", post(add_new_task))
        .route("/update-description/{task_id}", post(update_description))
        .layer(TraceLayer::new_for_http());
}

async fn root() -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut minijinja_env = Environment::new();
    minijinja_env.set_loader(path_loader("assets"));
    let template = minijinja_env.get_template("index.html.j2")?;
    return Ok(Html(
        template.render(context! { tasks => task_repo.get_all_tasks()? })?,
    ));
}

#[derive(Deserialize)]

struct AddNewTaskInput {
    priority: char,
    description: String,
}

async fn add_new_task(Form(task_desc): Form<AddNewTaskInput>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let task = Task::new(task_desc.priority, &task_desc.description)?;
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}

async fn set_done(Path(task_id): Path<TaskId>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut task = task_repo.get_task(task_id)?;
    task.completed = true;
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}

async fn set_pending(Path(task_id): Path<TaskId>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut task = task_repo.get_task(task_id)?;
    task.completed = false;
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}

async fn increase_priority(Path(task_id): Path<TaskId>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut task = task_repo.get_task(task_id)?;
    task.increase_priority();
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}

async fn lower_priority(Path(task_id): Path<TaskId>) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut task = task_repo.get_task(task_id)?;
    task.lower_priority();
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}

#[derive(Deserialize)]

struct UpdateDescriptionInput {
    task_description: String,
}

async fn update_description(
    Path(task_id): Path<TaskId>,
    Form(task_description): Form<UpdateDescriptionInput>,
) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(None)?;

    let mut task = task_repo.get_task(task_id)?;
    task.description = String::from(task_description.task_description.trim());
    task_repo.persist_task(&task)?;

    return Ok(Redirect::to("/"));
}
