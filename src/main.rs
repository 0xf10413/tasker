use axum::Form;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::post;
use axum::{Router, routing::get};
use minijinja::path_loader;
use minijinja::{Environment, context};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Set up dummy task list
    let mut task_list = TaskList::new();
    task_list.add('A', "some important task");
    task_list.add('B', "some less important task");
    task_list.add('Z', "some forgettable task");

    let shared_state = Arc::new(AppState {
        task_list: task_list.into(),
    });

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/toggle-done/{task_id}", post(toggle_done))
        .route("/add-new-task", post(add_new_task))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    // TODO: remove `unwrap` here
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

type TaskId = usize;

#[derive(Serialize, PartialEq)]
struct Task {
    id: TaskId,
    priority: char,
    description: String,
    completed: bool,
}

impl Eq for Task {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.completed, other.completed) {
            // If one of them is completed, it comes after
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
            // If both are unfinished, compare with priority then description
            (false, false) => self
                .priority
                .cmp(&other.priority)
                .then(self.description.cmp(&other.description)),
            // If both are finished, compare with completion date then description
            // TODO: completion date is not implemented, keeping prio for now
            (true, true) => self
                .priority
                .cmp(&other.priority)
                .then(self.description.cmp(&other.description)),
        }
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Serialize)]

struct TaskList {
    tasks: Vec<Task>,
}

impl TaskList {
    fn new() -> Self {
        TaskList { tasks: Vec::new() }
    }

    fn add(&mut self, priority: char, description: &str) {
        if priority < 'A' || priority > 'Z' {
            panic!() // TODO: remove panic!
        }
        self.tasks.push(Task {
            id: self.tasks.len(),
            priority: priority,
            description: String::from(description),
            completed: false,
        });

        // Always ensure we are sorted
        self._sort();
    }

    fn toggle_task_status(&mut self, task_id: TaskId) {
        let task = &mut self.tasks[task_id];
        task.completed = !task.completed;

        // Always ensure we are sorted
        self._sort();
    }

    // Sort internally and recompute task IDs
    fn _sort(&mut self) {
        self.tasks.sort();

        for (index, task) in self.tasks.iter_mut().enumerate() {
            task.id = index
        }
    }
}

#[derive(Deserialize)]
struct ToggleTaskInput {
    task_id: TaskId,
}
#[derive(Deserialize)]

struct AddNewTaskInput {
    priority: char,
    description: String,
}

struct AppState {
    task_list: Mutex<TaskList>,
}

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut minijinja_env = Environment::new();
    minijinja_env.set_loader(path_loader("assets"));
    let template = minijinja_env.get_template("index.html.j2").unwrap();
    return Html(
        template
            .render(context! { task_list => state.task_list })
            .unwrap(),
    );
}

async fn toggle_done(
    Path(task_id): Path<ToggleTaskInput>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let mut task_list = state.task_list.lock().unwrap();
    task_list.toggle_task_status(task_id.task_id);

    return Redirect::to("/");
}
async fn add_new_task(
    State(state): State<Arc<AppState>>,
    Form(task_desc): Form<AddNewTaskInput>,
) -> Redirect {
    let mut task_list = state.task_list.lock().unwrap();
    task_list.add(task_desc.priority, &task_desc.description);

    return Redirect::to("/");
}
