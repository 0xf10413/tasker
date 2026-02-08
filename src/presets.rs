use serde::Serialize;

pub type PresetTaskId = i64;
pub type PresetId = i64;

#[derive(Serialize, Debug)]
pub struct PresetTask {
    pub id: PresetTaskId,    // -1 if never persisted, ID in DB otherwise
    pub preset_id: PresetId, // always valid
    pub priority: char,
    pub description: String,
}

#[derive(Debug)]
pub enum PresetTaskError {
    // Tried to set priority to a value outside of A..Z
    PriorityNotInRangeError(char),
}

impl std::fmt::Display for PresetTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PriorityNotInRangeError(c) => write!(f, "Priority {} is invalid", c),
        }
    }
}

impl PresetTask {
    // Creates a brand new, never-persisted-before PresetTask
    pub fn new(
        priority: char,
        description: &str,
        preset_id: PresetId,
    ) -> Result<PresetTask, PresetTaskError> {
        if !priority.is_ascii_uppercase() {
            return Err(PresetTaskError::PriorityNotInRangeError(priority));
        }
        Ok(PresetTask {
            id: -1,
            preset_id,
            priority,
            description: description.into(),
        })
    }
}

#[derive(Serialize, Debug)]
pub struct Preset {
    pub id: PresetId, // -1 if never persisted, ID in DB otherwise
    pub name: String,
    pub tasks: Vec<PresetTask>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_usage() {
        let task =
            PresetTask::new('A', "Some nice task", 42).expect("Task creation should not fail");

        assert_eq!(task.id, -1); // Unpersisted tasks should have a special ID
        assert_eq!(task.priority, 'A');
        assert_eq!(task.description, "Some nice task");
    }
}
