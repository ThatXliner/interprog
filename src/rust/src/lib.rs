// TODO: Multithreading
use serde::{Deserialize, Serialize};
use serde_json::ser::to_string as to_json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskType {
    pub name: String,
    pub progress: Progress,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "name")]
pub enum Progress {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "finished")]
    Finished,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "in_progress")]
    // TODO: Infinitely nested
    InProgress { done: usize, total: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskManager {
    pub tasks: Vec<TaskType>,
    _totals: HashMap<String, usize>,
    pub task_counter: usize,
}

impl TaskManager {
    fn output(&self) {
        println!("{}", to_json(&self.tasks).expect("Should never happen"));
    }
    pub fn new() -> TaskManager {
        TaskManager {
            tasks: Vec::new(),
            _totals: HashMap::new(),
            task_counter: 0,
        }
    }

    pub fn set_total(&mut self, new_total: usize) {
        let task = &mut self.tasks[self.task_counter];

        match &task.progress {
            Progress::InProgress { done: _, total: _ } => {
                task.progress = Progress::InProgress {
                    done: 0,
                    total: new_total,
                };
            }
            _ => {
                panic!("Only bar-type tasks may have a total");
            }
        }
    }

    pub fn add_task(&mut self, name: String, total: Option<usize>) {
        self.tasks.push(TaskType {
            name: name.clone(),
            progress: Progress::Pending,
        });
        if let Some(t) = total {
            self._totals.insert(name, t);
        }
    }

    pub fn start(&mut self) {
        let task = &mut self.tasks[self.task_counter];
        if self._totals.contains_key(&task.name) {
            task.progress = Progress::InProgress {
                done: 0,
                total: self._totals[&task.name],
            };
        } else {
            task.progress = Progress::Running;
        }
        self.output();
    }

    pub fn increment(&mut self, by: usize, silent: bool) {
        let task = &mut self.tasks[self.task_counter];
        if let Progress::InProgress { done, total } = task.progress {
            if done >= total {
                if !silent {
                    panic!("Maxed out");
                }
                return;
            }
            task.progress = Progress::InProgress {
                done: done + by,
                total,
            };
            self.output();
        } else if !silent {
            panic!("Task is a spinner");
        }
    }

    pub fn finish(&mut self) {
        let task = &mut self.tasks[self.task_counter];
        task.progress = Progress::Finished;
        self._totals.remove(&task.name);
        self.task_counter += 1;
        self.output();
    }

    pub fn error(&mut self, message: String) {
        let task = &mut self.tasks[self.task_counter];
        task.progress = Progress::Error { message };
        self._totals.remove(&task.name);
        self.task_counter += 1;
        self.output();
    }
}

#[cfg(test)]
mod tests {
    use crate::TaskManager;

    #[test]
    fn it_works() {
        let mut manager = TaskManager::new();
        println!("Hi");
        manager.add_task("name".into(), None);
        manager.start();
        manager.finish();
    }
}
