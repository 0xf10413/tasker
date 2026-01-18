use axum::Form;
use axum::extract::Path;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::post;
use axum::{Router, routing::get};
use minijinja::path_loader;
use minijinja::{Environment, context};
use rusqlite::Connection;
use rusqlite::Row;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;

const SQLITE_URL: &str = "./tasks.db";

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let conn = Connection::open(SQLITE_URL).unwrap();
    let _ = conn
        .execute(
            "
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            priority TEXT NOT NULL,
            description TEXT NOT NULL,
            completed INTEGER NOT NULL
        )
        ",
            (),
        )
        .unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/set-pending/{task_id}", post(set_pending))
        .route("/set-done/{task_id}", post(set_done))
        .route("/increase-priority/{task_id}", post(increase_priority))
        .route("/lower-priority/{task_id}", post(lower_priority))
        .route("/add-new-task", post(add_new_task))
        .layer(TraceLayer::new_for_http());

    // TODO: remove `unwrap` here
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

type TaskId = i64;

#[derive(Serialize, Debug)]
struct Task {
    id: TaskId,     // -1 if never persisted, ID in DB otherwise
    priority: char, // TODO: change into its own struct
    description: String,
    completed: bool,
}

impl Task {
    fn new(priority: char, description: &str) -> Task {
        if priority < 'A' || priority > 'Z' {
            panic!() // TODO: remove panic!
        }
        return Task {
            id: -1,
            priority: priority,
            description: String::from(description),
            completed: false,
        };
    }

    fn increase_priority(&mut self) {
        match self.priority {
            'A' => (), // Do nothing if the priority is already maxed out
            _ => self.priority = std::char::from_u32(self.priority as u32 - 1).unwrap(),
        }
    }

    fn lower_priority(&mut self) {
        match self.priority {
            'Z' => (), // Do nothing if the priority is already at the minimum value
            _ => self.priority = std::char::from_u32(self.priority as u32 + 1).unwrap(),
        }
    }
}

struct TaskRepo {
    conn: Connection,
}

impl TaskRepo {
    fn new(path: Option<&str>) -> Self {
        TaskRepo {
            conn: Connection::open(match path {
                Some(p) => p,
                None => SQLITE_URL,
            })
            .unwrap(),
        }
    }

    fn task_from_row(row: &Row) -> Task {
        Task {
            id: row.get_unwrap(0),
            priority: row.get_unwrap::<usize, String>(1).chars().nth(0).unwrap(),
            description: row.get_unwrap(2),
            completed: row.get_unwrap(3),
        }
    }

    fn get_all_tasks(&mut self) -> Vec<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "
            SELECT id, priority, description, completed FROM tasks
            ORDER BY completed ASC, priority ASC, description ASC
            ",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |row| Ok(Self::task_from_row(row)))
            .unwrap();
        return Vec::from_iter(rows.into_iter().map(|result| result.unwrap()));
    }

    fn get_task(&mut self, task_id: TaskId) -> Task {
        let mut stmt = self
            .conn
            .prepare(
                "
            SELECT id, priority, description, completed FROM tasks
            WHERE id = ?
            ",
            )
            .unwrap();

        let mut rows = stmt.query([task_id]).unwrap();
        let row = rows.next().unwrap().unwrap();

        return Self::task_from_row(row);
    }

    fn persist_task(&mut self, task: &Task) {
        if task.id < 0 {
            // New task, need to insert
            let mut stmt = self
                .conn
                .prepare(
                    "
            INSERT INTO tasks (priority, description, completed)
            VALUES (:priority, :description, :completed)
            ",
                )
                .unwrap();

            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed};
            let _ = stmt.execute(params).unwrap();
        } else {
            // Existing task, need to update
            let mut stmt = self
                .conn
                .prepare(
                    "
            UPDATE tasks SET
            priority = :priority, description = :description, completed = :completed
            WHERE id = :id",
                )
                .unwrap();
            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed, ":id": task.id};
            let _ = stmt.execute(params).unwrap();
        }
    }
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
