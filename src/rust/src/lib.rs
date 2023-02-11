//! Inter-process progress reports.
//!
//! This module contains a `TaskManager` which you should instantiate once and reuse. It will schedule and output the tasks that you set to be running, queued, finished, and/or errored
use serde::{Deserialize, Serialize};
use serde_json::ser::to_string as to_json;
use std::collections::HashMap;
use std::io::{self, Write};
/// Represents a task.
///
/// TODO: Maybe change API to use macro/factory and a `.add` method on the `TaskManager`?
#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    /// The name of a task
    pub name: String,
    /// The current status of a task
    /// This field is named "progress"
    /// since the serialization will have a "status"
    /// field for the name.
    /// Making this field flattened (so there's no `progress` key in the first place) with `#[serde(flatten)]` would
    /// make it be annoying for other implementations to deserialize output in a type-safe manner
    pub progress: Status,
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
    /// The total field exists in this `Pending` vairant since if we use, say
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
            to_json(&self.tasks.values().collect::<Vec<_>>()).expect("Should never happen")
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

    pub fn set_task_total(&mut self, task_name: &str, new_total: usize) {
        let task = &mut self.tasks.get_mut(task_name).expect("Task does not exist");

        if let Status::Pending { total: _ } = &task.progress {
            task.progress = Status::Pending {
                total: Some(new_total),
            }
        } else {
            panic!("Task already started");
        }
    }
    pub fn set_total(&mut self, new_total: usize) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.set_task_total(&task_name, new_total);
    }

    pub fn add_task(&mut self, name: &str, total: Option<usize>) {
        self.tasks.insert(
            name.to_string(),
            Task {
                name: name.to_string(),
                progress: Status::Pending { total },
            },
        );
        self.task_list.push(name.to_string());
    }

    pub fn start_task(&mut self, task_name: &str) {
        let task = &mut self.tasks.get_mut(task_name).expect("Task does not exist");
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
            panic!("Task is already running")
        }
        self.output();
    }
    pub fn start(&mut self) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.start_task(&task_name);
    }

    pub fn increment_task(&mut self, task_name: &str, by: usize, silent: bool) {
        let task = &mut self.tasks.get_mut(task_name).expect("Task does not exist");
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
                    if !silent {
                        panic!("Maxed out");
                    }
                    return;
                }
                task.progress = Status::InProgress {
                    done: done + by,
                    total: *total,
                    // subtasks: None,
                };
            }
            Status::Running if !silent => {
                panic!("Task is a spinner");
            }
            _ => {
                if !silent {
                    panic!("Task already finished");
                }
                return;
            }
        }
        self.output();
    }
    pub fn increment(&mut self, by: usize, silent: bool) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.increment_task(&task_name, by, silent);
    }

    pub fn finish_task(&mut self, task_name: &str) {
        let task = &mut self.tasks.get_mut(task_name).expect("Task does not exist");
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
    }
    pub fn finish(&mut self) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.finish_task(&task_name);
    }

    pub fn error_task(&mut self, task_name: &str, message: &str) {
        let task = &mut self.tasks.get_mut(task_name).expect("Task does not exist");
        task.progress = Status::Error {
            message: message.to_string(),
        };
        self.task_counter += 1;
        self.output();
    }
    pub fn error(&mut self, message: &str) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.error_task(&task_name, message);
    }
}
impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use crate::TaskManager;

    #[test]
    fn it_works() {
        let mut manager = TaskManager::new();
        println!("Hi");
        manager.add_task("name", None);
        manager.start();
        manager.finish();
    }
    #[test]
    fn real_example() {
        let mut manager = TaskManager::new();
        manager.add_task("Log in", None);
        manager.start();
        manager.finish();
        let CLASSSES = vec!["English", "History", "Science", "Math"];
        for class in &CLASSSES {
            manager.add_task(&(format!("Scraping {class}")), None);
            manager.set_task_total(&(format!("Scraping {class}")), 4);
        }
        for _ in 0..4 {
            for class in &CLASSSES {
                manager.increment_task(&(format!("Scraping {class}")), 1, false);
            }
        }
    }
}
