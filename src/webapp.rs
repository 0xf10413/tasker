use crate::task::Task;
use crate::task::TaskId;

use crate::task_repo::TaskRepo;
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, Redirect},
    routing::{get, post},
};
use minijinja::{Environment, context, path_loader};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

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

async fn root() -> Html<String> {
    let mut task_repo = TaskRepo::new(None);

    let mut minijinja_env = Environment::new();
    minijinja_env.set_loader(path_loader("assets"));
    let template = minijinja_env.get_template("index.html.j2").unwrap();
    return Html(
        template
            .render(context! { tasks => task_repo.get_all_tasks() })
            .unwrap(),
    );
}

#[derive(Deserialize)]

struct AddNewTaskInput {
    priority: char,
    description: String,
}

async fn add_new_task(Form(task_desc): Form<AddNewTaskInput>) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let task = Task::new(task_desc.priority, &task_desc.description);
    task_repo.persist_task(&task);

    return Redirect::to("/");
}

async fn set_done(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let mut task = task_repo.get_task(task_id);
    task.completed = true;
    task_repo.persist_task(&task);

    return Redirect::to("/");
}

async fn set_pending(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let mut task = task_repo.get_task(task_id);
    task.completed = false;
    task_repo.persist_task(&task);

    return Redirect::to("/");
}

async fn increase_priority(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let mut task = task_repo.get_task(task_id);
    task.increase_priority();
    task_repo.persist_task(&task);

    return Redirect::to("/");
}

async fn lower_priority(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let mut task = task_repo.get_task(task_id);
    task.lower_priority();
    task_repo.persist_task(&task);

    return Redirect::to("/");
}

#[derive(Deserialize)]

struct UpdateDescriptionInput {
    task_description: String,
}

async fn update_description(
    Path(task_id): Path<TaskId>,
    Form(task_description): Form<UpdateDescriptionInput>,
) -> Redirect {
    let mut task_repo = TaskRepo::new(None);

    let mut task = task_repo.get_task(task_id);
    task.description = task_description.task_description; // TODO: trim
    task_repo.persist_task(&task);

    return Redirect::to("/");
}
