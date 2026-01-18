use axum::Form;
use axum::extract::Path;
use axum::response::Html;
use axum::response::Redirect;
use axum::routing::post;
use axum::{Router, routing::get};
use minijinja::path_loader;
use minijinja::{Environment, context};
use rusqlite::Connection;
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
    id: TaskId,
    priority: char, // TODO: change into its own struct
    description: String,
    completed: bool,
}

impl Task {
    fn increase_priority(&mut self){
        match self.priority {
            'A' => (), // Do nothing if the priority is already maxed out
            _ => self.priority = std::char::from_u32(self.priority as u32 - 1).unwrap()
        }
    }

    fn lower_priority(&mut self){
        match self.priority {
            'Z' => (),// Do nothing if the priority is already at the minimum value
            _ => self.priority = std::char::from_u32(self.priority as u32 + 1).unwrap()
        }
    }
}

struct TaskRepo {
    conn: Connection,
}

impl TaskRepo {
    fn new() -> Self {
        TaskRepo {
            conn: Connection::open(SQLITE_URL).unwrap(),
        }
    }

    fn add(&mut self, priority: char, description: &str) {
        if priority < 'A' || priority > 'Z' {
            // TODO: move into Task directly
            panic!() // TODO: remove panic!
        }
        let _ = self
            .conn
            .execute(
                "
            INSERT INTO tasks (priority, description, completed) VALUES (?, ?, ?)
        ",
                (String::from(priority), description, false),
            )
            .unwrap();
    }

    fn get_all_tasks(&mut self) -> Vec<Task> {
        let mut stmt: rusqlite::Statement<'_> = self
            .conn
            .prepare(
                "
            SELECT id, priority, description, completed FROM tasks
            ORDER BY completed ASC, priority ASC, description ASC
            ",
            )
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: row.get_unwrap(0),
                    priority: row.get_unwrap::<usize, String>(1).chars().nth(0).unwrap(),
                    description: row.get_unwrap(2),
                    completed: row.get_unwrap(3),
                })
            })
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

        return Task {
            id: row.get_unwrap(0),
            priority: row.get_unwrap::<usize, String>(1).chars().nth(0).unwrap(),
            description: row.get_unwrap(2),
            completed: row.get_unwrap(3),
        };
    }

    fn persist_task(&mut self, task: &Task) {
        let mut stmt: rusqlite::Statement<'_>;
        let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed, ":id": task.id};
        if task.id < 0 {
            // New task, need to insert
            todo!("Cannot persist new tasks")
        } else {
            stmt = self
                .conn
                .prepare(
                    "
            UPDATE tasks SET
            priority = :priority, description = :description, completed = :completed
            WHERE id = :id",
                )
                .unwrap();
        }

        let _ = stmt.execute(params).unwrap();
    }

    fn lower_priority(&mut self, task_id: TaskId) {
        let mut task = self.get_task(task_id);
        task.lower_priority();
        self.persist_task(&task)
    }

    fn increase_priority(&mut self, task_id: TaskId) {
        let mut task = self.get_task(task_id);
        task.increase_priority();
        self.persist_task(&task)
    }

    fn set_done(&mut self, task_id: TaskId) {
        let mut task = self.get_task(task_id);
        task.completed = true;
        self.persist_task(&task)
    }

    fn set_pending(&mut self, task_id: TaskId) {
        let mut task = self.get_task(task_id);
        task.completed = false;
        self.persist_task(&task)
    }

}

#[derive(Deserialize)]

struct AddNewTaskInput {
    priority: char,
    description: String,
}

async fn root() -> Html<String> {
    let mut task_list = TaskRepo::new();

    let mut minijinja_env = Environment::new();
    minijinja_env.set_loader(path_loader("assets"));
    let template = minijinja_env.get_template("index.html.j2").unwrap();
    return Html(
        template
            .render(context! { tasks => task_list.get_all_tasks() })
            .unwrap(),
    );
}

async fn add_new_task(Form(task_desc): Form<AddNewTaskInput>) -> Redirect {
    let mut task_list = TaskRepo::new();
    task_list.add(task_desc.priority, &task_desc.description);

    return Redirect::to("/");
}

async fn set_done(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_list = TaskRepo::new();
    task_list.set_done(task_id);

    return Redirect::to("/");
}

async fn set_pending(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_list = TaskRepo::new();
    task_list.set_pending(task_id);

    return Redirect::to("/");
}

async fn increase_priority(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_list = TaskRepo::new();
    task_list.increase_priority(task_id);
    return Redirect::to("/");
}

async fn lower_priority(Path(task_id): Path<TaskId>) -> Redirect {
    let mut task_list = TaskRepo::new();
    task_list.lower_priority(task_id);
    return Redirect::to("/");
}
