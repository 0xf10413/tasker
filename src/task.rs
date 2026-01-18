use serde::Serialize;

pub type TaskId = i64;

#[derive(Serialize, Debug)]
pub struct Task {
    pub id: TaskId,     // -1 if never persisted, ID in DB otherwise
    pub priority: char, // TODO: change into its own struct
    pub description: String,
    pub completed: bool,
}

impl Task {
    pub fn new(priority: char, description: &str) -> Task {
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

    pub fn increase_priority(&mut self) {
        match self.priority {
            'A' => (), // Do nothing if the priority is already maxed out
            _ => self.priority = std::char::from_u32(self.priority as u32 - 1).unwrap(),
        }
    }

    pub fn lower_priority(&mut self) {
        match self.priority {
            'Z' => (), // Do nothing if the priority is already at the minimum value
            _ => self.priority = std::char::from_u32(self.priority as u32 + 1).unwrap(),
        }
    }
}
