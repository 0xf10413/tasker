use serde::Serialize;

pub type TaskId = i64;

#[derive(Serialize, Debug)]
pub struct Task {
    pub id: TaskId, // -1 if never persisted, ID in DB otherwise
    pub priority: char,
    pub description: String,
    pub completed: bool,
}

pub enum TaskError {
    // Tried to set priority to a value outside of A..Z
    PriorityNotInRangeError(char),
}

impl Task {
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
