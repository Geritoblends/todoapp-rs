use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::{bson};
use thiserror::{Error as ThisError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Priority {
    Low,
    Regular,
    Urgent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(rename = "_id")]
    id: ObjectId,
    title: String,
    priority: Priority,
    completed: bool,
    created_at: NaiveDateTime
}

impl Task {

    pub fn new(id: ObjectId, title: &str, priority: Priority, created_at: NaiveDateTime) -> Self {
        Self {
            id,
            title: title.to_string(),
            priority,
            completed: false,
            created_at
        }
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
