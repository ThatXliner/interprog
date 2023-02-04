// TODO: Multithreading
use serde::{Deserialize, Serialize};
use serde_json::ser::to_string as to_json;
use std::collections::HashMap;
use std::io::{self, Write};
#[derive(Serialize, Deserialize, Debug)]
pub struct TaskType {
    pub name: String,
    pub progress: Progress,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Progress {
    #[serde(rename = "pending")]
    Pending { total: Option<usize> },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "in_progress")]
    InProgress {
        done: usize,
        total: usize,
        // TODO: Actual subtasks
        subtasks: Option<TaskManager>,
    },
}

#[derive(Serialize, Deserialize, Debug)]

pub struct TaskManager {
    pub tasks: HashMap<String, TaskType>,
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

    pub fn subtask_manager() -> Self {
        Self {
            tasks: HashMap::new(),
            task_list: Vec::new(),
            task_counter: 0,
            silent: true,
        }
    }

    pub fn set_task_total(&mut self, task_name: &str, new_total: usize) {
        let task = &mut self.tasks.get_mut(task_name).unwrap();

        match &task.progress {
            Progress::InProgress {
                done: _,
                total: _,
                subtasks: _,
            } => {
                task.progress = Progress::InProgress {
                    done: 0,
                    total: new_total,
                    subtasks: None,
                };
            }
            // Should we mutate `total` this instead?
            Progress::Pending { total: _ } => {
                task.progress = Progress::Pending {
                    total: Some(new_total),
                }
            }
            _ => {
                panic!("Only iterative tasks may have a total");
            }
        }
    }
    pub fn set_total(&mut self, new_total: usize) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.set_task_total(&task_name, new_total);
    }

    pub fn add_task(&mut self, name: &str, total: Option<usize>) {
        self.tasks.insert(
            name.to_string(),
            TaskType {
                name: name.to_string(),
                progress: Progress::Pending { total },
            },
        );
        self.task_list.push(name.to_string());
    }

    pub fn start_task(&mut self, task_name: &str) {
        let task = &mut self.tasks.get_mut(task_name).unwrap();
        match &task.progress {
            Progress::Pending { total } => match total {
                None => task.progress = Progress::Running,
                Some(total) => {
                    task.progress = Progress::InProgress {
                        done: 0,
                        total: *total,
                        subtasks: None,
                    };
                }
            },
            _ => {
                panic!("Task is already running")
            }
        }
        self.output();
    }
    pub fn start(&mut self) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.start_task(&task_name);
    }

    pub fn increment_task(&mut self, task_name: &str, by: usize, silent: bool) {
        let task = &mut self.tasks.get_mut(task_name).unwrap();
        // Never started before?
        if let Progress::Pending { total: Some(total) } = &task.progress {
            task.progress = Progress::InProgress {
                done: 1,
                total: *total,
                subtasks: None,
            };
            self.output();
        } else if let Progress::InProgress {
            done,
            total,
            subtasks: _,
        } = &task.progress
        {
            if done >= total {
                if !silent {
                    panic!("Maxed out");
                }
                return;
            }
            task.progress = Progress::InProgress {
                done: done + by,
                total: *total,
                subtasks: None,
            };
            self.output();
        } else if !silent {
            panic!("Task is a spinner");
        }
    }
    pub fn increment(&mut self, by: usize, silent: bool) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.increment_task(&task_name, by, silent);
    }

    pub fn finish_task(&mut self, task_name: &str) {
        let task = &mut self.tasks.get_mut(task_name).unwrap();
        if let Progress::InProgress {
            done: _,
            total: _,
            subtasks: Some(ref mut subtasks),
        } = task.progress
        {
            for task in &subtasks.task_list.clone() {
                subtasks.finish_task(&task);
            }
        }

        task.progress = Progress::Finished;
        self.task_counter += 1;
        self.output();
    }
    pub fn finish(&mut self) {
        let task_name: String = self.task_list[self.task_counter].clone();
        self.finish_task(&task_name);
    }

    pub fn error_task(&mut self, task_name: &str, message: &str) {
        let task = &mut self.tasks.get_mut(task_name).unwrap();
        task.progress = Progress::Error {
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
}
