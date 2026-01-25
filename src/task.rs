use serde::Serialize;

pub type TaskId = i64;

#[derive(Serialize, Debug)]
pub struct Task {
    pub id: TaskId, // -1 if never persisted, ID in DB otherwise
    pub priority: char,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug)]
pub enum TaskError {
    // Tried to set priority to a value outside of A..Z
    PriorityNotInRangeError(char),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PriorityNotInRangeError(c) => write!(f, "Priority {} is invalid", c),
        }
    }
}

impl Task {
    // Creates a brand new, never-persisted-before Task
    pub fn new(priority: char, description: &str) -> Result<Task, TaskError> {
        if priority < 'A' || priority > 'Z' {
            return Err(TaskError::PriorityNotInRangeError(priority));
        }
        return Ok(Task {
            id: -1,
            priority: priority,
            description: String::from(description),
            completed: false,
        });
    }

    pub fn increase_priority(&mut self) {
        match self.priority {
            'A' => (), // Do nothing if the priority is already maxed out
            _ => {
                self.priority = std::char::from_u32(self.priority as u32 - 1)
                    .expect("Priority should be convertible safely")
            }
        }
    }

    pub fn lower_priority(&mut self) {
        match self.priority {
            'Z' => (), // Do nothing if the priority is already at the minimum value
            _ => {
                self.priority = std::char::from_u32(self.priority as u32 + 1)
                    .expect("Priority should be convertible safely")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_usage() {
        let mut task = Task::new('A', "Some nice task").expect("Task creation should not fail");

        assert_eq!(task.id, -1); // Unpersisted tasks should have a special ID
        assert_eq!(task.completed, false); // Newly created tasks are not done
        assert_eq!(task.priority, 'A');
        assert_eq!(task.description, "Some nice task");

        task.lower_priority();
        assert_eq!(task.priority, 'B');

        task.increase_priority();
        assert_eq!(task.priority, 'A');
    }

    #[test]
    fn increase_max_priority_lower_min_priority() {
        let mut urgent_task =
            Task::new('A', "Some urgent task").expect("Task creation should not fail");
        let mut unimportant_task =
            Task::new('Z', "Some unimportant task").expect("Task creation should not fail");

        urgent_task.increase_priority();
        assert_eq!(urgent_task.priority, 'A'); // No failure, but no change either

        unimportant_task.lower_priority();
        assert_eq!(unimportant_task.priority, 'Z'); // No failure, but no change either
    }

    #[test]
    fn new_task_out_of_range() {
        let new_task_result = Task::new('4', "Some task with an invalid priority");

        assert!(new_task_result.is_err(), "Task creation should fail")
    }
}
