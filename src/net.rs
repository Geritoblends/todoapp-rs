use todo_app::{Priority, Task};
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};

#[derive(Deserialize, Decode)]
pub enum Command {
    NewTask{title: String, priority: Priority},
    PendingTasks,
    DoneTasks,
    MarkTaskDone(i32),
    EditTaskTitle{task_id: i32, new_title: String},
    EditTaskPriority{task_id: i32, priority: Priority},
    QueryTaskById(i32),
}

#[derive(Deserialize, Decode)]
pub struct ClientRequest {
    commands: Vec<Command>,
}

impl ClientRequest {
    pub fn get_commands(&self) -> Vec<Command> {
        self.commands.clone();
    }
}

#[derive(Serialize, Encode)]
pub enum CommandResponseValue {
    NewTask(Task),
    PendingTasks(Vec<Task>),
    DoneTasks(Vec<Task>),
    MarkTaskDone,
    EditTaskTitle,
    EditTaskPriority,
    QueryTaskById(Task),
}

#[derive(Serialize, Encode)]
pub enum CommandResponse {
    Success(CommandResponseValue),
    Error(String),
}

#[derive(Serialize, Encode)]
pub struct ServerResponse {
    payload: Vec<CommandResponse>
}
