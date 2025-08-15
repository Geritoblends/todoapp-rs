use serde::{Serialize, Deserialize};
use mongodb::bson::{doc, Document, DateTime};
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

impl Priority {

    pub fn to_string(&self) -> String {
        match self {
            Priority::Low => "Low".to_string(),
            Priority::Regular => "Regular".to_string(),
            Priority::Urgent => "Urgent".to_string(),
        }
    } 

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(rename = "_id", with = "bson::serde_helpers::hex_string_as_object_id")]
    id: String,
    title: String,
    priority: Priority,
    completed: bool,
    created_at: DateTime
}

impl Task {

    pub fn new(id: &str, title: &str, priority: Priority, created_at: DateTime) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            priority,
            completed: false,
            created_at
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_title(&self) -> String {
        self.title.to_string()
    }

    pub fn format(&self) -> String {
        format!("[{}]: {}", self.priority.to_string(), self.title)
    }

}

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    NewTask{title: String, priority: Priority},
    PendingTasks,
    DoneTasks,
    MarkTaskDone(String),
    EditTaskTitle{task_id: String, new_title: String},
    EditTaskPriority{task_id: String, priority: Priority},
    QueryTaskById(String),
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
    MarkTaskDone(Task),
    EditTaskTitle(Task),
    EditTaskPriority(Task),
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
