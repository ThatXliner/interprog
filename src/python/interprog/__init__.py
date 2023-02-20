"""Inter-process progress reports made easy

This API should be self-explanatory. See the specification
docs if you have questions
"""
import json
from dataclasses import dataclass, field
import dataclasses
from typing import Dict, List, Optional, Union, Literal


def _status(msg):
    def wrapper(cls):
        def status(self) -> str:
            return msg

        cls.status = property(status)

    return wrapper


@dataclass
class PendingStatus:
    total: Optional[int] = field(default=None)

    @property
    def status(self) -> Literal["pending"]:
        return "pending"


@dataclass
class RunningStatus:
    @property
    def status(self) -> Literal["running"]:
        return "running"


@dataclass
class FinishedStatus:
    @property
    def status(self) -> Literal["finished"]:
        return "finished"


@dataclass
class ErrorStatus:
    message: str

    @property
    def status(self) -> Literal["error"]:
        return "error"


@dataclass
class InProgressStatus:
    done: int
    total: int

    @property
    def status(self) -> Literal["in_progress"]:
        return "in_progress"

    # subtasks: List["TaskManager"]


Status = Union[
    PendingStatus, RunningStatus, FinishedStatus, ErrorStatus, InProgressStatus
]


@dataclass
class Task:
    name: str
    # TODO: Recursive tasks
    progress: Status

    def set_name(self, new_name: str):
        self.name = new_name
        return self

    def set_total(self, new_total: int):
        self.progress.total = new_total
        return self


JsonOutput = List[Task]


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


@dataclass
class TaskManager:
    tasks: Dict[str, Task] = field(default_factory=dict)
    task_list: JsonOutput = field(default_factory=list)
    task_counter: int = 0
    silent: bool = False

    def _output(self) -> None:
        if self.silent:
            return
        # TODO: A buffer that flushes at an interval
        print(json.dumps(self.task_list, cls=EnhancedJSONEncoder))

    def _current_task(self) -> Task:
        return self.tasks[self.task_list[self.task_counter]]

    def add_task(self, task: Task) -> None:
        """Enqueue a task"""
        if task.name in self.tasks:
            raise RuntimeError("Task already exists")
        self.tasks[task.name] = task
        self.task_list.append(task.name)
        self._output()

    def start_task(self, task_name: str) -> None:
        """Start the next task"""
        task = self.tasks[task_name]
        if not isinstance(task.progress, PendingStatus):
            raise RuntimeError("Task is already running")
        if task.progress.total is not None:
            task.progress = InProgressStatus(done=0, total=task.progress.total)
        else:
            task.progress = RunningStatus()
        self._output()

    def increment_task(self, task_name: str, by: int = 1, silent: bool = True) -> None:
        """Increment a bar task"""
        task = self.tasks[task_name]
        if isinstance(task.progress, PendingStatus):
            if task.progress.total is None:
                raise RuntimeError("Task is a spinner")
            task.progress = InProgressStatus(done=1, total=task.progress.total)
        elif isinstance(task.progress, InProgressStatus):
            if task.progress.done >= task.progress.done:
                if silent:
                    return
                raise RuntimeError("Maxed out")
            task.progress.done += by

        elif isinstance(task.progress, RunningStatus):
            raise RuntimeError("Task is a spinner")
        else:
            raise RuntimeError("Task already finished")
        self._output()

    def finish_task(self, task_name: str) -> None:
        """Mark a task as finished"""
        task = self.tasks[task_name]
        task.progress = FinishedStatus()
        self.task_counter += 1
        self._output()

    def error_task(self, task_name: str, message: str) -> None:
        """Mark a task as errored with a reason"""
        task = self.tasks[task_name]
        task.progress = ErrorStatus(message)
        self.task_counter += 1
        self._output()


__version__ = "0.3.0"
