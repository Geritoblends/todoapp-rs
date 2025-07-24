use todo_app::{Priority, Task};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Clone)]
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

impl ClientRequest {
    pub fn get_commands(&self) -> &[Command] {
        &self.commands
    }
}

#[derive(Serialize)]
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
pub enum CommandResponse {
    Success(CommandResponseValue),
    Error(String),
}

#[derive(Serialize)]
pub struct ServerResponse {
    payload: Vec<CommandResponse>
}

impl ServerResponse {
    pub fn new(payload: Vec<CommandResponse>) -> Self  {
        Self {
            payload,
        }
    }
}
