"""Inter-process progress reports made easy

This API should be self-explanatory. See the specification
docs if you have questions
"""
import json
from dataclasses import dataclass, field
from typing import Dict, List, Optional, TypedDict, Union, Literal


class PendingStatus(TypedDict):
    status: Literal["pending"]
    total: Optional[int]


class RunningStatus(TypedDict):
    status: Literal["running"]


class FinishedStatus(TypedDict):
    status: Literal["finished"]


class ErrorStatus(TypedDict):
    status: Literal["error"]
    message: str


class InProgressStatus(TypedDict):
    status: Literal["in_progress"]
    done: int
    total: int
    # subtasks: List["TaskManager"]


Status = Union[
    PendingStatus, RunningStatus, FinishedStatus, ErrorStatus, InProgressStatus
]


class Task(TypedDict):
    name: str
    # TODO: Recursive tasks
    progress: Status


JsonOutput = List[Task]


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
        print(json.dumps(self.task_list))

    def _current_task(self) -> Task:
        return self.tasks[self.task_list[self.task_counter]]

    def set_task_total(self, task_name: str, new_total: int) -> None:
        """Set total of the current bar task"""
        task = self.tasks[task_name]
        if task["progress"]["status"] != "pending":
            raise RuntimeError("Task already started")
        task["progress"]["total"] = new_total
        self._output()

    def add_task(self, name: str, total: Optional[int] = None) -> None:
        """Enqueue a task"""
        self.tasks.append(
            {
                "name": name,
                "progress": {
                    "status": "pending",
                    **({"total": total} if total is not None else {}),
                },
            }
        )
        self._output()

    def start_task(self, task_name: str) -> None:
        """Start the next task"""
        task = self.tasks[task_name]
        if task["progress"]["status"] != "pending":
            raise RuntimeError("Task is already running")
        if "total" in task["progress"] and task["progress"]["total"] is not None:
            task["progress"]["done"] = 0
            task["progress"]["status"] = "in_progress"
        else:
            task["progress"]["status"] = "running"
        self._output()

    def increment_task(self, task_name: str, by: int = 1, silent: bool = True) -> None:
        """Increment a bar task"""
        task = self.tasks[task_name]
        if task["progress"]["status"] == "pending":
            if "total" not in task["progress"] or task["progress"]["total"] is None:
                raise RuntimeError("Task is a spinner")
            task["progress"] = {
                "status": "in_progress",
                "done": 1,
                "total": task["progress"]["total"],
            }
        elif task["progress"]["status"] == "in_progress":
            if task["progress"]["done"] >= task["progress"]["done"]:
                if silent:
                    return
                raise RuntimeError("Maxed out")
            task["progress"] = {
                "status": "in_progress",
                "done": task["done"] + by,
                "total": task["progress"]["total"],
            }

        elif task["progress"]["status"] == "running":
            raise RuntimeError("Task is a spinner")
        else:
            raise RuntimeError("Task already finished")
        self._output()

    def finish_task(self, task_name: str) -> None:
        """Mark a task as finished"""
        task = self.tasks[task_name]
        task["progress"] = {"status": "finished"}
        self.task_counter += 1
        self._output()

    def error_task(self, task_name: str, message: str) -> None:
        """Mark a task as errored with a reason"""
        task = self.tasks[task_name]
        task["progress"] = {"status": "error", "message": message}
        self.task_counter += 1
        self._output()


__version__ = "0.3.0"
