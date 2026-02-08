use std::sync::Arc;

use crate::presets::PresetTask;
use crate::sql_connection_factory::SqlConnectionFactory;
use crate::task::Task;
use crate::task::TaskError;
use crate::task::TaskId;

use crate::task_repo::{TaskRepo, TaskRepoError};
use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::http::Response;
use axum::http::StatusCode;
use axum::{
    Form, Router,
    extract::Path,
    response::{Html, IntoResponse, Redirect, Result},
    routing::{get, post},
};
use minijinja::value::ViaDeserialize;
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
            Self::PresetTaskError { original_error } => original_error.to_string(),
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
        // Home page
        .route("/", get(root))
        // Basic task handling
        .route("/add-new-task", post(add_new_task))
        .route("/flag-pending/{task_id}", post(flag_pending))
        .route("/flag-completed/{task_id}", post(flag_completed))
        .route("/increase-priority/{task_id}", post(increase_priority))
        .route("/lower-priority/{task_id}", post(lower_priority))
        .route("/update-description/{task_id}", post(update_description))
        // Advanced manipulation
        .route("/task-cleanup", post(task_cleanup))
        .route("/rename-project", post(rename_project))
        // Presets
        .route("/preset", post(add_new_preset))
        .route("/preset/{preset_name}", get(get_preset))
        .route(
            "/preset/{preset_name}/add-new-preset-task",
            post(add_new_preset_task),
        )
        .route("/preset/{preset_name}/inject", post(inject_preset))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

// Fixes printing of projects in the UI.
fn projectify(project: ViaDeserialize<Option<String>>) -> String {
    match project.as_deref() {
        Some(s) => s.into(),
        None => "".into(),
    }
}

fn render<S: Serialize>(template: &str, context: S) -> Result<Html<String>, TaskRepoError> {
    let mut env = Environment::new();
    env.set_loader(path_loader("assets"));
    env.add_filter("projectify", projectify);
    let template = env.get_template(template)?;
    Ok(Html(template.render(context)?))
}

#[derive(Deserialize)]
struct ProjectSelect {
    project: Option<String>,
}

async fn root(
    State(state): State<AppState>,
    Query(project): Query<ProjectSelect>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);
    let all_tasks = task_repo.get_all_tasks(project.project.as_deref())?;
    let all_projects = task_repo.get_all_projects()?;
    let all_preset_names = task_repo.get_all_preset_names()?;

    render(
        "index.html.j2",
        context! { tasks => all_tasks, projects => all_projects, current_project => project.project, preset_names => all_preset_names },
    )
}

#[derive(Deserialize)]
struct AddNewTaskInput {
    priority: char,
    description: String,
    project: Option<String>,
}

async fn add_new_task(
    State(state): State<AppState>,
    Form(task): Form<AddNewTaskInput>,
) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let task = Task::new(task.priority, &task.description, task.project.as_deref())?;
    task_repo.persist_task(&task)?;

    Ok(Redirect::to("/"))
}

async fn flag_completed(
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

#[derive(Deserialize)]
struct RenameProjectInput {
    current_project_name: String,
    new_project_name: String,
}

async fn rename_project(
    State(state): State<AppState>,
    Form(input): Form<RenameProjectInput>,
) -> Result<Redirect> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    task_repo.rename_project(&input.current_project_name, &input.new_project_name)?;

    Ok(Redirect::to("/"))
}

#[derive(Deserialize)]
struct AddNewPresetInput {
    preset_name: String,
}

async fn add_new_preset(
    State(state): State<AppState>,
    Form(preset): Form<AddNewPresetInput>,
) -> Result<Redirect, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);
    task_repo.add_preset(&preset.preset_name)?;

    let redirection_url = format!("/preset/{}", preset.preset_name);
    Ok(Redirect::to(&redirection_url))
}

async fn get_preset(
    State(state): State<AppState>,
    Path(preset_name): Path<String>,
) -> Result<Html<String>, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);
    let preset = task_repo.get_preset(&preset_name)?;

    render("preset.html.j2", context! { preset => preset})
}

#[derive(Deserialize)]
struct AddNewPresetTaskInput {
    task_priority: char,
    task_description: String,
}

async fn add_new_preset_task(
    State(state): State<AppState>,
    Path(preset_name): Path<String>,
    Form(preset_task): Form<AddNewPresetTaskInput>,
) -> Result<Redirect, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let preset_id = task_repo.get_preset_id_from_preset_name(&preset_name)?;

    let preset_task = PresetTask::new(
        preset_task.task_priority,
        &preset_task.task_description,
        preset_id,
    )?;
    task_repo.persist_preset_task(preset_task)?;

    let redirection_url = format!("/preset/{}", preset_name);
    Ok(Redirect::to(&redirection_url))
}

async fn inject_preset(
    State(state): State<AppState>,
    Path(preset_name): Path<String>,
) -> Result<Redirect, TaskRepoError> {
    let mut task_repo = TaskRepo::new(state.connection_factory);

    let preset = task_repo.get_preset(&preset_name)?;
    for preset_task in preset.tasks {
        let task = Task::new(
            preset_task.priority,
            &preset_task.description,
            Some(&preset_name),
        )?;
        task_repo.persist_task(&task)?
    }

    Ok(Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use crate::sql_connection_factory::tests::TempDirSqliteConnectionFactory;

    use super::*;
    use axum::http::{self, Request, header::LOCATION};
    use http_body_util::BodyExt;
    use tower::Service;

    async fn add_new_task(
        app: &mut Router,
        priority: char,
        description: &str,
        project: Option<&str>,
    ) {
        let mut form_text: String = format!("priority={priority}&description={description}");
        if let Some(project) = project {
            form_text = format!("{form_text}&project={project}");
        }

        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/add-new-task")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(form_text))
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
    async fn full_basic_flow() {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new().unwrap());
        TaskRepo::new(connection_factory.clone()).init_db().unwrap();

        let mut app = build_app(AppState { connection_factory });

        // Add new task
        add_new_task(&mut app, 'B', "SomeTask", None).await;

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

        // Flag as completed
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/flag-completed/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;

        // Ensure task is flagged as completed
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

        let mut app = build_app(AppState { connection_factory });

        // Add new task
        add_new_task(&mut app, 'B', "SomeTask", None).await;
        add_new_task(&mut app, 'A', "SomeImportantTask", None).await;
        add_new_task(&mut app, 'C', "SomeNotImportantTask", None).await;

        // Flag some of them as completed
        for i in 1..=2 {
            let response = app
                .call(
                    Request::builder()
                        .method(http::Method::POST)
                        .uri(format!("/flag-completed/{i}"))
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
        assert!(!parsed_body.contains("SomeTask")); // Completed => removed
        assert!(!parsed_body.contains("SomeImportantTask")); // Completed => removed
        assert!(parsed_body.contains("SomeNotImportantTask")); // Pending => kept
    }

    #[tokio::test]
    async fn tasks_and_projects() {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new().unwrap());
        TaskRepo::new(connection_factory.clone()).init_db().unwrap();

        let mut app = build_app(AppState { connection_factory });

        // Add new task with or without projects
        add_new_task(&mut app, 'B', "SomeTask", None).await;
        add_new_task(&mut app, 'B', "SomeOtherTask", Some("project1")).await;

        // Ensure it appears in the output
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(parsed_body.contains("project1"));

        // Rename project
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/rename-project")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(
                        "current_project_name=project1&new_project_name=project2",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/");

        // Ensure new name appears in the output
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(parsed_body.contains("project2"));
    }

    #[tokio::test]
    async fn presets() {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new().unwrap());
        TaskRepo::new(connection_factory.clone()).init_db().unwrap();

        let mut app = build_app(AppState { connection_factory });

        // Add new preset
        let form_text: String = "preset_name=preset1".to_string();
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/preset")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(form_text))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/preset/preset1");

        // Check it out
        let response = app
            .call(
                Request::builder()
                    .uri("/preset/preset1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;
        assert!(parsed_body.contains("preset1"));

        // Add a new preset task
        let form_text: String = "task_priority=A&task_description=my_new_description".to_string();
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/preset/preset1/add-new-preset-task")
                    .header(
                        http::header::CONTENT_TYPE,
                        mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
                    )
                    .body(Body::from(form_text))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/preset/preset1");

        // Check it out
        let response = app
            .call(
                Request::builder()
                    .uri("/preset/preset1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let parsed_body = parse_body(response).await;
        assert!(parsed_body.contains("my_new_description"));

        // Nothing should be on the home page yet
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(!parsed_body.contains("my_new_description"));

        // Inject preset
        let response = app
            .call(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/preset/preset1/inject")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers().get(LOCATION).unwrap(), "/");

        // And now the task should be injected
        let parsed_body = get_main_page_body(&mut app).await;
        assert!(parsed_body.contains("my_new_description"));
    }
}
