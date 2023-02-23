class InterprogError(Exception):
    """Raised by Interprog."""


class TaskAlreadyExistsError(InterprogError):
    pass


class TaskAlreadyRunningError(InterprogError):
    pass


class InvalidTaskTypeError(InterprogError):
    pass


class MaxedOutTaskError(InterprogError):
    pass
