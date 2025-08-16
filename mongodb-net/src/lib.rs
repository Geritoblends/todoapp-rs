use serde::{Serialize, Deserialize};
use mongodb::bson::doc;
use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use mongodb::{bson};
use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
pub struct DateTimeOutOfRangeError;

impl fmt::Display for DateTimeOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unix timestamp exceeds the i64 integer range")
    }
}

impl std::error::Error for DateTimeOutOfRangeError {}

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
pub struct TaskDocument {
    #[serde(rename = "_id")]
    id: ObjectId,
    title: String,
    priority: Priority,
    completed: bool,
    created_at: i64
}

impl TaskDocument {
    
    pub fn new(id: &str, title: &str, priority: Priority, completed: bool, created_at: i64) -> Result<Self, bson::oid::Error> {
        let id = ObjectId::from_str(id)?;
        Ok(Self {
            id,
            title: title.to_string(),
            priority,
            completed,
            created_at
        })
    }

    pub fn as_task(&self) -> Result<Task, DateTimeOutOfRangeError> {
        let timestamp = match NaiveDateTime::from_timestamp_millis(self.created_at) {
            Some(timestamp) => timestamp,
            None => return Err(DateTimeOutOfRangeError),
        };
        Ok(Task::new(&self.id.to_hex(), &self.title, self.priority, timestamp))
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    id: String,
    title: String,
    priority: Priority,
    completed: bool,
    created_at: NaiveDateTime
}

impl Task {

    pub fn new(id: &str, title: &str, priority: Priority, created_at: NaiveDateTime) -> Self {
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

    pub fn as_document(&self) -> Result<TaskDocument, bson::oid::Error> {
        let doc = TaskDocument::new(&self.id, &self.title, self.priority, self.completed, self.created_at.timestamp_millis())?;
        Ok(doc)
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
