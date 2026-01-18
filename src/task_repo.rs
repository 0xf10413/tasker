use rusqlite::Connection;
use rusqlite::Row;
use rusqlite::named_params;

use crate::task::Task;
use crate::task::TaskId;

const SQLITE_URL: &str = "./tasks.db";

pub struct TaskRepo {
    conn: Connection,
}

impl TaskRepo {
    pub fn new(path: Option<&str>) -> Self {
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

    pub fn init_db(&mut self) {
        let _ = self
            .conn
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
    }

    pub fn get_all_tasks(&mut self) -> Vec<Task> {
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

    pub fn get_task(&mut self, task_id: TaskId) -> Task {
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

    pub fn persist_task(&mut self, task: &Task) {
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
