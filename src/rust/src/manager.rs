//! The main structs that manages tasks and printing them
//!
//! Most methods have an `task` variant that
//! works on a specified task name instead of
//! the first unfinished task queued (FIFO).
use crate::{errors, Status, Task};
use serde::{Deserialize, Serialize};
use serde_json::ser::to_string as to_json_string;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub trait TaskManagerApi {
    // Hey... the &mut self...
    fn add_task(&mut self, task: Task) -> Result<(), errors::InterprogError>;
    fn start_task(&mut self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError>;
    fn increment_task(
        &mut self,
        task_name: impl AsRef<str>,
        by: usize,
    ) -> Result<(), errors::InterprogError>;
    fn finish_task(&mut self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError>;
    fn error_task(
        &mut self,
        task_name: impl AsRef<str>,
        message: impl Into<String>,
    ) -> Result<(), errors::InterprogError>;
    fn get_task(&mut self, task_name: impl AsRef<str>)
        -> Result<&mut Task, errors::InterprogError>;
}

/// Synchronous non-thread-safe implementation of the TaskManager API
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TaskManager {
    pub tasks: HashMap<String, Task>,
    pub task_list: Vec<String>,
    pub task_counter: usize,
    silent: bool,
}

impl TaskManager {
    #[inline]
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

    pub fn start(&mut self) -> Result<(), errors::InterprogError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::InterprogError::NonexistentTask)?
            .clone();
        self.start_task(&task_name)
    }
    pub fn increment(&mut self, by: usize) -> Result<(), errors::InterprogError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::InterprogError::NonexistentTask)?
            .clone();
        self.increment_task(&task_name, by)
    }
    pub fn finish(&mut self) -> Result<(), errors::InterprogError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::InterprogError::NonexistentTask)?
            .clone();
        self.finish_task(&task_name)
    }
    pub fn error(&mut self, message: impl Into<String>) -> Result<(), errors::InterprogError> {
        let task_name: String = self
            .task_list
            .get(self.task_counter)
            .ok_or(errors::InterprogError::NonexistentTask)?
            .clone();
        self.error_task(&task_name, message)
    }
}
impl TaskManagerApi for TaskManager {
    fn get_task(
        &mut self,
        task_name: impl AsRef<str>,
    ) -> Result<&mut Task, errors::InterprogError> {
        return Ok(self
            .tasks
            .get_mut(task_name.as_ref())
            .ok_or(errors::InterprogError::NonexistentTask)?);
    }
    fn add_task(&mut self, task: Task) -> Result<(), errors::InterprogError> {
        let name = task.name.clone();
        match self.tasks.entry(name.clone()) {
            Entry::Occupied(_) => return Err(errors::InterprogError::TaskAlreadyExists),
            Entry::Vacant(entry) => entry.insert(task),
        };
        self.task_list.push(name);
        Ok(())
    }

    fn start_task(&mut self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError> {
        let task = self.get_task(task_name)?;
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
            return Err(errors::InterprogError::TaskAlreadyStarted);
        }
        self.output();
        Ok(())
    }

    fn increment_task(
        &mut self,
        task_name: impl AsRef<str>,
        by: usize,
    ) -> Result<(), errors::InterprogError> {
        let task = self.get_task(task_name)?;
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
                    return Err(errors::InterprogError::TaskAlreadyFinished);
                }
                // TODO: If incrementing makes it full, do we consider finished?
                task.progress = Status::InProgress {
                    done: done + by,
                    total: *total,
                    // subtasks: None,
                };
            }
            Status::Running | Status::Pending { total: None } => {
                return Err(errors::InterprogError::InvalidTaskType)
            }
            Status::Finished | Status::Error { message: _ } => {
                return Err(errors::InterprogError::TaskAlreadyFinished)
            }
        }
        self.output();
        Ok(())
    }

    fn finish_task(&mut self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError> {
        let task = self.get_task(task_name)?;
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

    fn error_task(
        &mut self,
        task_name: impl AsRef<str>,
        message: impl Into<String>,
    ) -> Result<(), errors::InterprogError> {
        let task = self.get_task(task_name)?;
        task.progress = Status::Error {
            message: message.into(),
        };
        self.task_counter += 1;
        self.output();
        Ok(())
    }
}
impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AtomicTaskManagerRef {
    tasks: HashMap<String, Task>,
    task_list: Vec<String>,
    task_counter: usize,
    silent: bool,
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomicTaskManager {
    inner: Arc<Mutex<AtomicTaskManagerRef>>, // pub tasks: HashMap<String, Task>,
                                             // pub task_list: Vec<String>,
                                             // pub task_counter: usize,
                                             // silent: bool,
}

impl AtomicTaskManager {
    #[inline]
    fn output(&self) {
        let inner = self.inner.lock().unwrap();
        if inner.silent {
            return;
        }
        println!(
            "{}",
            to_json_string(&inner.tasks.values().collect::<Vec<_>>()).expect("Should never happen")
        );
        io::stdout().flush().unwrap();
    }
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AtomicTaskManagerRef {
                tasks: HashMap::new(),
                task_list: Vec::new(),
                task_counter: 0,
                silent: false,
            })),
        }
    }

    pub fn add_task(&self, task: Task) -> Result<(), errors::InterprogError> {
        {
            let name = task.name.clone();
            let mut inner = self.inner.lock().unwrap();
            match inner.tasks.entry(name.clone()) {
                Entry::Occupied(_) => return Err(errors::InterprogError::TaskAlreadyExists),
                Entry::Vacant(entry) => entry.insert(task),
            };
            inner.task_list.push(name);
        }
        self.output();
        Ok(())
    }

    pub fn start_task(&self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError> {
        {
            let mut inner = self.inner.lock().unwrap();
            let task = inner
                .tasks
                .get_mut(task_name.as_ref())
                .ok_or(errors::InterprogError::NonexistentTask)?;
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
                return Err(errors::InterprogError::TaskAlreadyStarted);
            }
        }
        self.output();
        Ok(())
    }
    pub fn start(&self) -> Result<(), errors::InterprogError> {
        let task_name = {
            let inner = self.inner.lock().unwrap();
            inner
                .task_list
                .get(inner.task_counter)
                .ok_or(errors::InterprogError::NonexistentTask)?
                .clone()
        };
        self.start_task(&task_name)
    }

    pub fn increment_task(
        &self,
        task_name: impl AsRef<str>,
        by: usize,
    ) -> Result<(), errors::InterprogError> {
        {
            let mut inner = self.inner.lock().unwrap();
            let task = inner
                .tasks
                .get_mut(task_name.as_ref())
                .ok_or(errors::InterprogError::NonexistentTask)?;
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
                        return Err(errors::InterprogError::TaskAlreadyFinished);
                    }
                    // TODO: If incrementing makes it full, do we consider finished?
                    task.progress = Status::InProgress {
                        done: done + by,
                        total: *total,
                        // subtasks: None,
                    };
                }
                Status::Running | Status::Pending { total: None } => {
                    return Err(errors::InterprogError::InvalidTaskType)
                }
                Status::Finished | Status::Error { message: _ } => {
                    return Err(errors::InterprogError::TaskAlreadyFinished)
                }
            }
        }
        self.output();
        Ok(())
    }
    pub fn increment(&self, by: usize) -> Result<(), errors::InterprogError> {
        let task_name = {
            let inner = self.inner.lock().unwrap();
            inner
                .task_list
                .get(inner.task_counter)
                .ok_or(errors::InterprogError::NonexistentTask)?
                .clone()
        };
        self.increment_task(&task_name, by)
    }

    pub fn finish_task(&self, task_name: impl AsRef<str>) -> Result<(), errors::InterprogError> {
        {
            let mut inner = self.inner.lock().unwrap();
            let task = inner
                .tasks
                .get_mut(task_name.as_ref())
                .ok_or(errors::InterprogError::NonexistentTask)?;
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
            inner.task_counter += 1;
        }
        self.output();
        Ok(())
    }
    pub fn finish(&self) -> Result<(), errors::InterprogError> {
        let task_name = {
            let inner = self.inner.lock().unwrap();
            inner
                .task_list
                .get(inner.task_counter)
                .ok_or(errors::InterprogError::NonexistentTask)?
                .clone()
        };
        self.finish_task(&task_name)
    }

    pub fn error_task(
        &self,
        task_name: impl AsRef<str>,
        message: impl Into<String>,
    ) -> Result<(), errors::InterprogError> {
        {
            let mut inner = self.inner.lock().unwrap();
            let task = inner
                .tasks
                .get_mut(task_name.as_ref())
                .ok_or(errors::InterprogError::NonexistentTask)?;
            task.progress = Status::Error {
                message: message.into(),
            };
            inner.task_counter += 1;
        }
        self.output();
        Ok(())
    }
    pub fn error(&self, message: impl Into<String>) -> Result<(), errors::InterprogError> {
        let task_name = {
            let inner = self.inner.lock().unwrap();
            inner
                .task_list
                .get(inner.task_counter)
                .ok_or(errors::InterprogError::NonexistentTask)?
                .clone()
        };
        self.error_task(&task_name, message)
    }
}
impl Default for AtomicTaskManager {
    fn default() -> Self {
        Self::new()
    }
}