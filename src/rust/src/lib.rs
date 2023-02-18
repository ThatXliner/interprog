//! Inter-process progress reports.
//!
//! This module contains a `TaskManager` which you should instantiate once and reuse. It will schedule and output the tasks that you set to be running, queued, finished, and/or errored
pub mod errors;

use serde::{Deserialize, Serialize};
use serde_json::ser::to_string as to_json_string;
use std::collections::HashMap;
use std::io::{self, Write};
/// Represents a task.
#[derive(Serialize, Deserialize, Debug)]
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
    pub fn new<S: Into<String>>(name: S) -> Self {
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
        return self;
    }
    /// Change the name
    pub fn name(mut self, new_name: String) -> Self {
        self.name = new_name;
        return self;
    }
}
/// Represents the status of a task.
///
/// There are 3 main states: Pending, Running, and finished.
/// But since there are 2 types of running (iterative or spinner) and 3 types of finished (success and error), thus 5 variants
#[derive(Serialize, Deserialize, Debug)]
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
/// The main struct that manages printing tasks
///
/// Most methods have an `task` variant that
/// works on a specified task name instead of
/// the first unfinished task (FIFO). This is to
/// account for the future, when we
/// actually support multithreading.
/// Yes, this struct is currently *not thread-safe*
/// (I think)
#[derive(Serialize, Deserialize, Debug)]
pub struct TaskManager {
    pub tasks: HashMap<String, Task>,
    pub task_list: Vec<String>,
    pub task_counter: usize,
    pub silent: bool,
}

impl TaskManager {
    fn output(&self) {
        if self.silent {
            return;
        }
        println!(
            "{}",
            to_json_string(&self.tasks.values().collect::<Vec<_>>()).expect("Should never happen")
        );
        io::stdout().flush().unwrap();
    }
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            task_list: Vec::new(),
            task_counter: 0,
            silent: false,
        }
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), errors::ManagerError> {
        let name = task.name.clone();
        self.tasks.insert(name.clone(), task);
        self.task_list.push(name.to_string());
        Ok(())
    }

    pub fn start_task(&mut self, task_name: &str) -> Result<(), errors::ManagerError> {
        let task = &mut self
            .tasks
            .get_mut(task_name)
            .ok_or(errors::ManagerError::NonexistentTask)?;
        if let Status::Pending { total } = &task.progress {
            match total {
                Some(total) => {
                    task.progress = Status::InProgress {
                        done: 0,
                        total: *total,
                        // subtasks: None,
                    };
                }
                None => task.progress = Status::Running,
            }
        } else {
            return Err(errors::ManagerError::TaskAlreadyStarted);
        }
        self.output();
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), errors::ManagerError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::ManagerError::NonexistentTask)?
            .clone();
        self.start_task(&task_name)
    }

    pub fn increment_task(
        &mut self,
        task_name: &str,
        by: usize,
    ) -> Result<(), errors::ManagerError> {
        let task = &mut self
            .tasks
            .get_mut(task_name)
            .ok_or(errors::ManagerError::NonexistentTask)?;
        // Never started before
        match &task.progress {
            Status::Pending { total: Some(total) } => {
                task.progress = Status::InProgress {
                    done: 1,
                    total: *total,
                    // subtasks: None,
                };
            }
            Status::InProgress {
                done,
                total,
                // subtasks: _,
            } => {
                if done >= total {
                    return Err(errors::ManagerError::TaskAlreadyFinished);
                }
                // TODO: If incrementing makes it full, do we consider finished?
                task.progress = Status::InProgress {
                    done: done + by,
                    total: *total,
                    // subtasks: None,
                };
            }
            Status::Running | Status::Pending { total: None } => {
                return Err(errors::ManagerError::InvalidTaskType)
            }
            Status::Finished | Status::Error { message: _ } => {
                return Err(errors::ManagerError::TaskAlreadyFinished)
            }
        }
        self.output();
        Ok(())
    }
    pub fn increment(&mut self, by: usize) -> Result<(), errors::ManagerError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::ManagerError::NonexistentTask)?
            .clone();
        self.increment_task(&task_name, by)
    }

    pub fn finish_task(&mut self, task_name: &str) -> Result<(), errors::ManagerError> {
        let task = &mut self
            .tasks
            .get_mut(task_name)
            .ok_or(errors::ManagerError::NonexistentTask)?;
        // TODO: Implement subtasks
        // if let Status::InProgress {
        //     done: _,
        //     total: _,
        //     subtasks: Some(ref mut subtasks),
        // } = task.progress
        // {
        //     for task in &subtasks.task_list.clone() {
        //         subtasks.finish_task(&task);
        //     }
        // }

        task.progress = Status::Finished;
        self.task_counter += 1;
        self.output();
        Ok(())
    }
    pub fn finish(&mut self) -> Result<(), errors::ManagerError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::ManagerError::NonexistentTask)?
            .clone();
        self.finish_task(&task_name)
    }

    pub fn error_task(
        &mut self,
        task_name: &str,
        message: &str,
    ) -> Result<(), errors::ManagerError> {
        let task = &mut self
            .tasks
            .get_mut(task_name)
            .ok_or(errors::ManagerError::NonexistentTask)?;
        task.progress = Status::Error {
            message: message.to_string(),
        };
        self.task_counter += 1;
        self.output();
        Ok(())
    }
    pub fn error(&mut self, message: &str) -> Result<(), errors::ManagerError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::ManagerError::NonexistentTask)?
            .clone();
        self.error_task(&task_name, message)
    }
}
impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use crate::{Task, TaskManager};

    #[test]
    fn it_works() {
        let mut manager = TaskManager::new();
        println!("Hi");
        manager.add_task(Task::new("name")).unwrap();
        manager.start().unwrap();
        manager.finish().unwrap();
    }
    #[test]
    fn real_example() {
        let mut manager = TaskManager::new();
        manager.add_task(Task::new("Log in")).unwrap();
        manager.start().unwrap();
        manager.finish().unwrap();
        let classes = vec!["English", "History", "Science", "Math"];
        for class in &classes {
            manager
                .add_task(Task::new(format!("Scraping {class}")).total(4))
                .unwrap();
        }
        for _ in 0..4 {
            for class in &classes {
                manager
                    .increment_task(&(format!("Scraping {class}")), 1)
                    .unwrap();
            }
        }
    }
}
