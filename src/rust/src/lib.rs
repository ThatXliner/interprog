//! Inter-process progress reports.
//!
//! This module contains a `TaskManager` which you should instantiate once and reuse. It will schedule and output the tasks that you set to be running, queued, finished, and/or errored
pub mod errors;
pub mod manager;
pub mod task;
pub use crate::manager::TaskManager;
pub use crate::task::{Status, Task};

#[cfg(test)]
mod tests {
    use crate::{Task, TaskManager};

    #[test]
    fn it_works() {
        let mut manager = TaskManager::new();
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
                    .increment_task(format!("Scraping {class}"), 1)
                    .unwrap();
            }
        }
    }
    #[test]
    fn static_names() {
        let mut manager = TaskManager::new();
        manager.add_task(Task::new("Log in")).unwrap();
        manager.start_task("Log in").unwrap();
        manager.finish().unwrap();
    }
}
