use serde::{Deserialize, Serialize};
/// Represents a task.
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Ord, PartialOrd, Clone)]
pub struct Task {
    /// The name of a task
    pub name: String,
    /// The current status of a task
    /// Making this field flattened (so there's no `progress` key in the first place) with `#[serde(flatten)]` would
    /// make it be annoying for other implementations to deserialize output in a type-safe manner
    ///
    /// ## Notes on the naming
    /// - This field is named "progress" since the serialization will have a "status" field for the name.
    /// - Naming this field `status` and renaming the current key for the status type to `type` (it's currently `status`) does not reflect that this whole key-value pair is about the progress of the task.
    pub progress: Status,
}
impl Task {
    pub fn new(name: impl Into<String>) -> Self {
        Task {
            name: name.into(),
            progress: Status::Pending { total: None },
        }
    }
    /// Change the total
    ///
    /// TODO: Subtasks
    pub fn total(mut self, new_total: usize) -> Self {
        self.progress = Status::Pending {
            total: Some(new_total),
        };
        self
    }
    /// Change the name
    pub fn name(mut self, new_name: impl Into<String>) -> Self {
        self.name = new_name.into();
        self
    }
}
impl From<String> for Task {
    fn from(name: String) -> Self {
        Task::new(name)
    }
}
/// Represents the status of a task.
///
/// There are 3 main states: Pending, Running, and finished.
/// But since there are 2 types of running (iterative or spinner) and 3 types of finished (success and error), thus 5 variants
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Ord, PartialOrd, Clone)]
#[serde(tag = "status")]
pub enum Status {
    /// Represents a pending task, waiting to be executed.
    ///
    /// The `total` field is optional. If it exists/is not null,
    /// it means the task is *iterative* and has a known end
    /// Otherwise, we assume it to be a spinner task.
    /// The total field exists in this `Pending` variant since if we use, say
    /// ```
    /// # use interprog::Status;
    /// # let X = 1;
    /// Status::InProgress{done: 0, total: X};
    /// ```
    /// to represent a pending task with a known total instead of
    /// ```
    /// # use interprog::Status;
    /// # let X = Some(1);
    /// Status::Pending{total: X};
    /// ```
    /// it is ambiguous whether or not the task has already
    /// started or not.
    #[serde(rename = "pending")]
    Pending { total: Option<usize> },
    /// Self-explanatory
    #[serde(rename = "error")]
    Error { message: String },
    /// Self-explanatory
    #[serde(rename = "finished")]
    Finished,
    /// Like `InProgress` but for non-iterative tasks (unknown total)
    #[serde(rename = "running")]
    Running,
    /// An **iterative task** (known end and/or subtasks)
    /// is running.
    ///
    /// `done` out of `total` tasks were finished
    /// The `subtasks` field is currently unused but will be
    /// in the future when we implement nested tasks
    /// TODO: implement subtasks
    #[serde(rename = "in_progress")]
    InProgress {
        done: usize,
        total: usize,
        // subtasks: Option<TaskManager>,
    },
}
