use axum::extract::State;
use axum::response::Html;
use axum::{Router, routing::get};
use minijinja::path_loader;
use minijinja::{Environment, context};
use serde::Serialize;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Set up dummy task list
    let mut task_list = TaskList::new();
    task_list.add(Task::new('A', "some important task"));
    task_list.add(Task::new('B', "some less important task"));
    task_list.add(Task::new('Z', "some forgettable task"));

    let shared_state = Arc::new(AppState {
        task_list: task_list,
    });

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http());

    // TODO: remove `unwrap` here
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}
#[derive(Serialize)]
struct Task {
    priority: char,
    description: String,
    completed: bool,
}

impl Task {
    fn new(priority: char, description: &str) -> Self {
        if priority < 'A' || priority > 'Z' {
            panic!() // TODO: remove panic!
        }
        return Self {
            priority: priority,
            description: String::from(description),
            completed: false,
        };
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

    fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }
}

struct AppState {
    task_list: TaskList,
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
