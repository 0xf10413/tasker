use std::sync::Arc;

use rusqlite::Row;
use rusqlite::named_params;
use rusqlite::params_from_iter;

use crate::presets::Preset;
use crate::presets::PresetId;
use crate::presets::PresetTask;
use crate::presets::PresetTaskError;
use crate::sql_connection_factory::SqlConnectionFactory;
use crate::task::Task;
use crate::task::TaskError;
use crate::task::TaskId;

pub struct TaskRepo {
    connection_factory: Arc<dyn SqlConnectionFactory>,
}

#[derive(Debug)]
pub enum TaskRepoError {
    Error { error: String },
    SqlError { original_error: rusqlite::Error },
    IoError { original_error: std::io::Error },
    JinjaError { original_error: minijinja::Error }, // TODO: this is not really a repo error...
    TaskError { original_error: TaskError },         // TODO: this is not really a repo error...
    PresetTaskError { original_error: PresetTaskError }, // TODO: this is not really a repo error...
}

impl From<rusqlite::Error> for TaskRepoError {
    fn from(value: rusqlite::Error) -> Self {
        TaskRepoError::SqlError {
            original_error: value,
        }
    }
}

impl From<std::io::Error> for TaskRepoError {
    fn from(value: std::io::Error) -> Self {
        TaskRepoError::IoError {
            original_error: value,
        }
    }
}

impl From<TaskError> for TaskRepoError {
    fn from(value: TaskError) -> Self {
        TaskRepoError::TaskError {
            original_error: value,
        }
    }
}

impl From<PresetTaskError> for TaskRepoError {
    fn from(value: PresetTaskError) -> Self {
        TaskRepoError::PresetTaskError {
            original_error: value,
        }
    }
}

impl TaskRepo {
    pub fn new(connection_factory: Arc<dyn SqlConnectionFactory>) -> TaskRepo {
        TaskRepo { connection_factory }
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
            project: {
                let raw: String = row.get(4)?;
                match raw.len() {
                    0 => None,
                    _ => Some(raw),
                }
            },
        })
    }

    fn preset_task_from_row(row: &Row) -> Result<PresetTask, TaskRepoError> {
        Ok(PresetTask {
            id: row.get(0)?,
            preset_id: row.get(1)?,
            priority: row
                .get::<usize, String>(2)?
                .chars()
                .nth(0)
                .ok_or(TaskRepoError::Error {
                    error: String::from("Priority in storage was empty"),
                })?,
            description: row.get(3)?,
        })
    }

    pub fn init_db(&mut self) -> Result<(), TaskRepoError> {
        let conn = self.connection_factory.open()?;
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                priority TEXT NOT NULL,
                description TEXT NOT NULL,
                completed INTEGER NOT NULL,
                project TEXT NOT NULL
            )
            ",
            (),
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS presets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )
            ",
            (),
        )?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS preset_tasks (
                id INTEGER PRIMARY KEY,
                preset_id INTEGER NOT NULL,
                priority TEXT NOT NULL,
                description TEXT NOT NULL,

                FOREIGN KEY(preset_id)
                REFERENCES presets(id)
                ON DELETE CASCADE
            )
            ",
            (),
        )?;

        Ok(())
    }

    pub fn get_all_tasks(
        &mut self,
        project_filter: Option<&str>,
    ) -> Result<Vec<Task>, TaskRepoError> {
        let conn = self.connection_factory.open()?;

        let mut stmt_sql: String =
            "SELECT id, priority, description, completed, project FROM tasks ".into();
        if project_filter.is_some() {
            stmt_sql.push_str("WHERE project = :project ");
        }
        stmt_sql.push_str("ORDER BY completed ASC, priority ASC, description ASC");

        let mut stmt = conn.prepare(&stmt_sql)?;
        let params = match project_filter {
            None => vec![],
            Some(s) => vec![s],
        };
        let rows = stmt.query_and_then(params_from_iter(params), Self::task_from_row)?;
        rows.into_iter().collect()
    }

    pub fn get_task(&mut self, task_id: TaskId) -> Result<Task, TaskRepoError> {
        let conn = self.connection_factory.open()?;
        let mut stmt = conn.prepare(
            "
            SELECT id, priority, description, completed, project FROM tasks
            WHERE id = ?
            ",
        )?;

        let mut rows = stmt.query([task_id])?;
        let row = rows.next()?.ok_or(TaskRepoError::Error {
            error: format!("Task {} not found in storage", task_id),
        })?;

        Self::task_from_row(row)
    }

    pub fn persist_task(&mut self, task: &Task) -> Result<(), TaskRepoError> {
        let conn = self.connection_factory.open()?;
        if task.id < 0 {
            // New task, need to insert
            let mut stmt = conn.prepare(
                "
            INSERT INTO tasks (priority, description, completed, project)
            VALUES (:priority, :description, :completed, :project)
            ",
            )?;

            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed, ":project": task.project.as_deref().unwrap_or("")};
            stmt.execute(params)?;
            Ok(())
        } else {
            // Existing task, need to update
            let mut stmt = conn.prepare(
                "
            UPDATE tasks SET
            priority = :priority, description = :description, completed = :completed
            WHERE id = :id",
            )?;
            let params = named_params! {":priority": String::from(task.priority), ":description": task.description, ":completed": task.completed, ":id": task.id};
            stmt.execute(params)?;
            Ok(())
        }
    }

    pub fn persist_preset_task(&mut self, preset_task: PresetTask) -> Result<(), TaskRepoError> {
        let conn = self.connection_factory.open()?;
        if preset_task.id < 0 {
            // New task, need to insert
            let mut stmt = conn.prepare(
                "
            INSERT INTO preset_tasks (preset_id, priority, description)
            VALUES (:preset_id, :priority, :description)
            ",
            )?;

            let params = named_params! {":preset_id": preset_task.preset_id, ":priority": String::from(preset_task.priority), ":description": preset_task.description};
            stmt.execute(params)?;
            Ok(())
        } else {
            Err(TaskRepoError::Error {
                error:
                    "Cannot persist a non-new preset task (i.e. preset task update not implemented)"
                        .into(),
            })
        }
    }

    pub fn cleanup(&mut self) -> Result<(), TaskRepoError> {
        let conn = self.connection_factory.open()?;

        conn.execute("DELETE FROM tasks WHERE completed", [])?;

        Ok(())
    }

    pub fn get_all_projects(&mut self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.connection_factory.open()?;
        let mut stmt = conn.prepare(
            "
            SELECT DISTINCT project FROM tasks
            WHERE project != ''
            ORDER BY project ASC
            ",
        )?;

        stmt.query_map([], |row| row.get::<_, String>(0))?.collect()
    }

    pub fn rename_project(
        &mut self,
        current_project_name: &str,
        new_project_name: &str,
    ) -> Result<(), TaskRepoError> {
        let conn = self.connection_factory.open()?;
        let mut stmt = conn.prepare(
            "
            UPDATE tasks
            SET project = :new_project_name
            WHERE project = :current_project_name
            ",
        )?;
        stmt.execute(named_params!{":current_project_name": current_project_name, ":new_project_name": new_project_name})?;

        Ok(())
    }

    pub fn add_preset(&mut self, new_preset_name: &str) -> Result<(), rusqlite::Error> {
        let conn = self.connection_factory.open()?;
        let mut stmt = conn.prepare(
            "
            INSERT INTO presets
            (name)
            VALUES (:new_preset_name)
            ",
        )?;
        stmt.execute(named_params! {":new_preset_name": new_preset_name})?;

        Ok(())
    }

    pub fn get_all_preset_names(&mut self) -> Result<Vec<String>, rusqlite::Error> {
        let conn = self.connection_factory.open()?;
        let mut stmt = conn.prepare(
            "
            SELECT DISTINCT name FROM presets
            ORDER BY name ASC
            ",
        )?;

        stmt.query_map([], |row| row.get::<_, String>(0))?.collect()
    }

    pub fn get_preset_id_from_preset_name(
        &mut self,
        preset_name: &str,
    ) -> Result<PresetId, TaskRepoError> {
        let conn = self.connection_factory.open()?;

        let mut stmt = conn.prepare("SELECT id FROM presets WHERE name = :preset_name")?;
        let mut rows = stmt.query(named_params! {":preset_name" : preset_name})?;
        let row = rows.next()?.ok_or(TaskRepoError::Error {
            error: format!("Preset {} not found in storage", preset_name),
        })?;
        Ok(row.get(0)?)
    }

    pub fn get_preset(&mut self, preset_name: &str) -> Result<Preset, TaskRepoError> {
        let conn = self.connection_factory.open()?;

        // Fetch preset ID
        let preset_id = self.get_preset_id_from_preset_name(preset_name)?;

        // Rebuild PresetTask collection
        let mut stmt = conn.prepare(
            "
            SELECT id, preset_id, priority, description FROM preset_tasks
            WHERE preset_id = :preset_id
            ",
        )?;
        let rows = stmt.query_and_then(
            named_params! {":preset_id": preset_id},
            Self::preset_task_from_row,
        )?;
        let tasks: Result<Vec<PresetTask>, TaskRepoError> = rows.into_iter().collect();

        // Bind together and return everything
        Ok(Preset {
            id: preset_id,
            name: preset_name.to_string(),
            tasks: tasks?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::sql_connection_factory::tests::TempDirSqliteConnectionFactory;

    use super::*;

    #[test]
    fn get_all_is_ordered() -> Result<(), TaskRepoError> {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new()?);
        let mut task_repo = TaskRepo::new(connection_factory);

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        assert!(task_repo.get_task(-1).is_err());

        task_repo.persist_task(&Task::new('B', "Medium task", None).unwrap())?;
        task_repo.persist_task(&Task::new('Z', "Unimportant task", None).unwrap())?;
        task_repo.persist_task(&Task::new('A', "Important task", None).unwrap())?;
        task_repo.persist_task(&Task::new('A', "Another important task", None).unwrap())?;

        let tasks = task_repo.get_all_tasks(None)?;
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
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new()?);
        let mut task_repo = TaskRepo::new(connection_factory);

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        task_repo.persist_task(&Task::new('B', "Medium task", None).unwrap())?;

        // Cheating a bit here, we can guess the ID of a task
        let mut retrieved_task = task_repo.get_task(1)?;

        // Should be unchanged
        assert_eq!(retrieved_task.priority, 'B');
        assert_eq!(retrieved_task.description, "Medium task");
        assert!(!retrieved_task.completed);

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
        assert!(retrieved_task.completed);

        Ok(())
    }

    #[test]
    fn cleanup() -> Result<(), TaskRepoError> {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new()?);
        let mut task_repo = TaskRepo::new(connection_factory);

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        task_repo.persist_task(&Task::new('C', "Some low importance task", None)?)?;

        // Pending tasks are spared
        task_repo.cleanup()?;
        let mut existing_task = task_repo.get_task(1)?;
        assert_eq!(existing_task.description, "Some low importance task");

        existing_task.completed = true;
        task_repo.persist_task(&existing_task)?;

        // Completed tasks are deleted
        task_repo.cleanup()?;
        assert!(task_repo.get_task(1).is_err());

        Ok(())
    }

    #[test]
    fn project_handling() -> Result<(), TaskRepoError> {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new()?);
        let mut task_repo = TaskRepo::new(connection_factory);

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        // By default, tasks do not pertain to any project
        task_repo.persist_task(&Task::new('B', "Medium task", None).unwrap())?;
        let global_task = task_repo.get_task(1)?;
        assert_eq!(global_task.project, None);

        let all_projects = task_repo.get_all_projects()?;
        assert_eq!(all_projects.len(), 0);

        // Tasks may have dedicated projects. Projects are created "on-the-fly"
        task_repo.persist_task(&Task::new('A', "Important task", "project".into()).unwrap())?;
        let project_task = task_repo.get_task(2)?;
        assert_eq!(project_task.project, Some("project".into()));

        let all_projects = task_repo.get_all_projects()?;
        assert_eq!(all_projects, ["project"]);

        // We can filter per project.
        let filtered_tasks = task_repo.get_all_tasks(Some("project"))?;
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].description, "Important task");

        // We can rename projects
        task_repo.rename_project("project", "project_2")?;
        let all_projects = task_repo.get_all_projects()?;
        assert_eq!(all_projects, ["project_2"]);
        let filtered_tasks_old_project = task_repo.get_all_tasks(Some("project"))?;
        assert_eq!(filtered_tasks_old_project.len(), 0);
        let filtered_tasks_new_project = task_repo.get_all_tasks(Some("project_2"))?;
        assert_eq!(filtered_tasks_new_project.len(), 1);
        assert_eq!(filtered_tasks_new_project[0].description, "Important task");

        Ok(())
    }

    #[test]
    fn presets() -> Result<(), TaskRepoError> {
        let connection_factory = Arc::new(TempDirSqliteConnectionFactory::new()?);
        let mut task_repo = TaskRepo::new(connection_factory);

        // Has to be called always to initialize schema
        task_repo.init_db()?;

        // Create a new preset
        task_repo.add_preset("new preset")?;

        // Fetch its ID
        let preset_id = task_repo.get_preset_id_from_preset_name("new preset")?;

        // Add a new preset task
        task_repo.persist_preset_task(PresetTask::new('A', "some description", preset_id)?)?;

        // We should be able to see it now
        let preset = task_repo.get_preset("new preset")?;
        assert_eq!(preset.tasks.len(), 1);
        let preset_task = &preset.tasks[0];
        assert_eq!(preset_task.description, "some description");
        assert_eq!(preset_task.priority, 'A');

        // No non-preset task should have been added
        assert_eq!(task_repo.get_all_tasks(None)?.len(), 0);

        // That's it.
        // Note that preset injection is not implemented here.
        Ok(())
    }
}
