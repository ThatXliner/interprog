use std::fmt;

#[derive(Debug)]
pub enum InterprogError {
    TaskAlreadyStarted,
    NonexistentTask,
    TaskAlreadyExists,
    InvalidTaskType,
    // XXX: Better naming
    TaskAlreadyFinished,
}
impl fmt::Display for InterprogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TaskAlreadyStarted => "Task has already been started/is running",
            Self::NonexistentTask => "The requested task does not exist",
            Self::TaskAlreadyExists => "Another task of the same name already exists",
            Self::InvalidTaskType => "The task is the wrong type for the requested operation",
            Self::TaskAlreadyFinished => "Task is already done",
        };
        write!(f, "{}", message)
    }
}
