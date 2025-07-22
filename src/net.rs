use todo_app::{Priority, Task};
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
#[repr(u8)]
pub enum Command {
    NewTask{title: String, priority: Priority},
    PendingTasks,
    DoneTasks,
    MarkTaskDone(i32),
    EditTaskTitle{task_id: i32, new_title: String},
    EditTaskPriority{task_id: i32, priority: Priority},
    QueryTaskById(i32),
}

#[derive(Deserialize)]
pub struct ClientRequest {
    commands: Vec<Command>,
}

#[derive(Serialize)]
#[repr(u8)]
pub enum CommandResponseValue {
    NewTask(Task),
    PendingTasks(Vec<Task>),
    DoneTasks(Vec<Task>),
    MarkTaskDone,
    EditTaskTitle,
    EditTaskPriority,
    QueryTaskById(Task),
}

#[derive(Serialize)]
#[repr(u8)]
pub enum CommandResponse {
    Success(CommandResponseValue),
    Error(String),
}

#[derive(Serialize)]
pub struct ServerResponse {
    payload: Vec<CommandResponse>
}

