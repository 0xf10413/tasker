use axum::extract::State;
use axum::response::Html;
use axum::{Router, routing::get};
use minijinja::path_loader;
use minijinja::{Environment, context};
use serde::Serialize;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let mut task_list = Vec::new();
    task_list.push(Task::new('A', "some important task"));
    task_list.push(Task::new('B', "some less task"));
    task_list.push(Task::new('Z', "some forgettable task"));

    let shared_state = Arc::new(AppState {
        task_list: task_list,
    });

    // build our application with a route
    let app = Router::new().route("/", get(root)).with_state(shared_state);

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

type TaskList = Vec<Task>;

struct AppState {
    task_list: TaskList,
}

async fn root(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut env = Environment::new();
    env.set_loader(path_loader("assets"));
    let template = env.get_template("index.html.j2").unwrap();
    return Html(
        template
            .render(context! { task_list => state.task_list })
            .unwrap(),
    );
}
