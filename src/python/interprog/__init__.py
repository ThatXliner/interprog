"""Inter-process progress reports made easy

This API should be self-explanatory. See the specification
docs if you have questions
"""
import json
from dataclasses import dataclass, field
import dataclasses
from typing import Dict, List, Optional, Union, Literal
import inspect
import copy


def _status(msg):
    def wrapper(cls):
        def status(self) -> str:
            return msg

        cls.status = property(status)

    return wrapper


@dataclass
class PendingStatus:
    total: Optional[int] = field(default=None)
    status: Literal["pending"] = field(default="pending")


@dataclass
class RunningStatus:
    status: Literal["running"] = field(default="running")


@dataclass
class FinishedStatus:
    status: Literal["finished"] = field(default="finished")


@dataclass
class ErrorStatus:
    message: str
    status: Literal["error"] = field(default="error")


@dataclass
class InProgressStatus:
    done: int
    total: int

    status: Literal["in_progress"] = field(default="in_progress")

    # subtasks: List["TaskManager"]


Status = Union[
    PendingStatus, RunningStatus, FinishedStatus, ErrorStatus, InProgressStatus
]


@dataclass
class Task:
    name: str
    # TODO: Recursive tasks
    progress: Status = field(default_factory=PendingStatus)

    def set_name(self, new_name: str):
        self.name = new_name
        return self

    def set_total(self, new_total: int):
        self.progress.total = new_total
        return self


JsonOutput = List[Task]


_YES_GEN = {"start_task", "increment_task", "finish_task", "error_task"}


def _add_non_task_methods(cls):
    for member_name, member in inspect.getmembers(copy.deepcopy(cls)):
        if member_name in _YES_GEN:

            def wrapper(self, *args, __name=member_name, **kwargs):
                print(f"Calling {__name}")
                getattr(self, __name)(
                    self.task_list[self.task_counter], *args, **kwargs
                )

            setattr(cls, member_name[:-5], wrapper)
    return cls


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


@dataclass
@_add_non_task_methods
class TaskManager:
    tasks: Dict[str, Task] = field(default_factory=dict)
    task_list: JsonOutput = field(default_factory=list)
    task_counter: int = 0
    silent: bool = False

    def _output(self) -> None:
        if self.silent:
            return
        # TODO: A buffer that flushes at an interval
        print(json.dumps(list(self.tasks.values()), cls=EnhancedJSONEncoder))

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
        if self.tasks[task_name].progress.total is not None:
            self.tasks[task_name].progress = InProgressStatus(
                done=0, total=self.tasks[task_name].progress.total
            )
        else:
            self.tasks[task_name].progress = RunningStatus()
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
