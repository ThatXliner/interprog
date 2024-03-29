use thiserror::Error;

// XXX: Better naming
#[derive(Error, Debug)]
pub enum InterprogError {
    #[error("Task has already been started/is running")]
    TaskAlreadyStarted,
    #[error("The requested task does not exist")]
    NonexistentTask,
    #[error("Another task of the same name already exists")]
    TaskAlreadyExists,
    #[error("The task is the wrong type for the requested operation")]
    InvalidTaskType,
    #[error("Task is already done")]
    TaskAlreadyFinished,
}
