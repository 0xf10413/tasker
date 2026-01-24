use rusqlite::Connection;
use rusqlite::Row;
use rusqlite::named_params;

use crate::task::Task;
use crate::task::TaskId;

const SQLITE_URL: &str = "./tasks.db";

pub struct TaskRepo {
    conn: Connection,
}

#[derive(Debug)]
pub enum TaskRepoError {
    Error { error: String },
    SqlError { original_error: rusqlite::Error },
    JinjaError { original_error: minijinja::Error }, // TODO: this is not really a repo error...
}

impl From<rusqlite::Error> for TaskRepoError {
    fn from(value: rusqlite::Error) -> Self {
        TaskRepoError::SqlError {
            original_error: value,
        }
    }
}

impl TaskRepo {
    pub fn new(connection: Option<rusqlite::Connection>) -> Result<Self, TaskRepoError> {
        Ok(TaskRepo {
            conn: match connection {
                Some(connection) => connection,
                None => Connection::open(SQLITE_URL)?,
            },
        })
    }

    fn task_from_row(row: &Row) -> Result<Task, TaskRepoError> {
        Ok(Task {
            id: row.get(0)?,
            priority: row
                .get::<usize, String>(1)?
                .chars()
                .nth(0)
                .ok_or(TaskRepoError::Error {
                    error: String::from("Priority in storage was empty"),
                })?,
            description: row.get(2)?,
            completed: row.get(3)?,
        })
    }

    pub fn init_db(&mut self) -> Result<(), TaskRepoError> {
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                priority TEXT NOT NULL,
                description TEXT NOT NULL,
                completed INTEGER NOT NULL
            )
            ",
            (),
        )?;
        Ok(())
    }

    pub fn get_all_tasks(&mut self) -> Result<Vec<Task>, TaskRepoError> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, priority, description, completed FROM tasks
            ORDER BY completed ASC, priority ASC, description ASC
            ",
        )?;
        let rows = stmt.query_and_then([], |row| Self::task_from_row(row))?;
        return rows.into_iter().collect();
    }

    pub fn get_task(&mut self, task_id: TaskId) -> Result<Task, TaskRepoError> {
        let mut stmt = self.conn.prepare(
            "
            SELECT id, priority, description, completed FROM tasks
            WHERE id = ?
            ",
        )?;

        let mut rows = stmt.query([task_id])?;
        let row = rows.next()?.ok_or(TaskRepoError::Error {
            error: format!("Task {} not found in storage", task_id),
        })?;

        return Self::task_from_row(row);
    }

    pub fn persist_task(&mut self, task: &Task) -> Result<(), TaskRepoError> {
        if task.id < 0 {
            // New task, need to insert
            let mut stmt = self.conn.prepare(
                "
            INSERT INTO tasks (priority, description, completed)
            VALUES (:priority, :description, :completed)
            ",
            )?;

            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed};
            stmt.execute(params)?;
            return Ok(());
        } else {
            // Existing task, need to update
            let mut stmt = self.conn.prepare(
                "
            UPDATE tasks SET
            priority = :priority, description = :description, completed = :completed
            WHERE id = :id",
            )?;
            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed, ":id": task.id};
            stmt.execute(params)?;
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_all_is_ordered() -> Result<(), TaskRepoError> {
        let conn = Connection::open_in_memory()?;
        let mut task_repo = TaskRepo::new(Some(conn))?;

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        assert!(task_repo.get_task(-1).is_err());

        task_repo.persist_task(&Task::new('B', "Medium task").unwrap())?;
        task_repo.persist_task(&Task::new('Z', "Unimportant task").unwrap())?;
        task_repo.persist_task(&Task::new('A', "Important task").unwrap())?;
        task_repo.persist_task(&Task::new('A', "Another important task").unwrap())?;

        let tasks = task_repo.get_all_tasks()?;
        assert_eq!(tasks.len(), 4);

        // Tasks should be sorted per decreasing priority, then alphabetically
        let tasks_descriptions: Vec<_> =
            tasks.iter().map(|task| task.description.clone()).collect();

        assert_eq!(
            tasks_descriptions,
            vec![
                "Another important task",
                "Important task",
                "Medium task",
                "Unimportant task"
            ]
        );

        Ok(())
    }

    #[test]
    fn persisting() -> Result<(), TaskRepoError> {
        let conn = Connection::open_in_memory()?;
        let mut task_repo = TaskRepo::new(Some(conn))?;

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        task_repo.persist_task(&Task::new('B', "Medium task").unwrap())?;

        // Cheating a bit here, we can guess the ID of a task
        let mut retrieved_task = task_repo.get_task(1)?;

        // Should be unchanged
        assert_eq!(retrieved_task.priority, 'B');
        assert_eq!(retrieved_task.description, "Medium task");
        assert_eq!(retrieved_task.completed, false);

        // Let's update it
        retrieved_task.lower_priority();
        retrieved_task.description = "A new description".into();
        retrieved_task.completed = true;

        task_repo.persist_task(&retrieved_task)?;

        // Let's retrieve it again
        let retrieved_task = task_repo.get_task(1)?;

        // Should have new fields
        assert_eq!(retrieved_task.priority, 'C');
        assert_eq!(retrieved_task.description, "A new description");
        assert_eq!(retrieved_task.completed, true);

        Ok(())
    }
}
