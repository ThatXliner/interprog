use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Task has already been started/is running")]
    TaskAlreadyStarted,
    #[error("The requested task does not exist")]
    NonexistentTask,
    #[error("The task is the wrong type for the requested operation")]
    InvalidTaskType,
    #[error("Task is already done")]
    MaxedOutTask,
}
