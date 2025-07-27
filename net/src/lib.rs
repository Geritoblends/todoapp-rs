use chrono::NaiveDateTime;
use sqlx::{Type, FromRow};
use serde::{Serialize, Deserialize};

#[derive(Type, Debug, Clone, Copy, Serialize, Deserialize)]
#[sqlx(type_name = "priority")]
#[sqlx(rename_all = "PascalCase")]
pub enum Priority {
    Low,
    Regular,
    Urgent,
}

#[derive(FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub title: String,
    pub priority: Priority,
    pub completed: bool,
    pub id: i32,
    pub created_at: NaiveDateTime,
}

impl Task {

    pub fn format(&self) -> String {
        format!("[{:?}]: {}", self.priority, self.title)
    }

}
#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    NewTask{title: String, priority: Priority},
    PendingTasks,
    DoneTasks,
    MarkTaskDone(i32),
    EditTaskTitle{task_id: i32, new_title: String},
    EditTaskPriority{task_id: i32, priority: Priority},
    QueryTaskById(i32),
}

#[derive(Deserialize, Serialize)]
pub struct ClientRequest {
    commands: Vec<Command>,
}

impl ClientRequest {
    pub fn get_commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn new(cmds: &[Command]) -> Self {
        Self {
            commands: cmds.to_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CommandResponseValue {
    NewTask(Task),
    PendingTasks(Vec<Task>),
    DoneTasks(Vec<Task>),
    MarkTaskDone,
    EditTaskTitle,
    EditTaskPriority,
    QueryTaskById(Task),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CommandResponse {
    Success(CommandResponseValue),
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct ServerResponse {
    payload: Vec<CommandResponse>
}

impl ServerResponse {
    pub fn new(payload: &[CommandResponse]) -> Self  {
        Self {
            payload: payload.to_vec(),
        }
    }

    pub fn unwrap(&self) -> Vec<CommandResponse> {
        self.payload.clone()
    }

}
