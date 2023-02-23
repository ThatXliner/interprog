use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Task has already been started/is running")]
    TaskAlreadyStarted,
    #[error("The requested task does not exist")]
    NonexistentTask,
    #[error("The task is the wrong type for the requested operation")]
    InvalidTaskType,
    #[error("Another task of the same name already exists")]
    TaskAlreadyExists,
    // XXX: Better naming
    #[error("Task is already done")]
    TaskAlreadyFinished,
}
