use chrono::{NaiveDateTime, Utc};
use mongodb::{Client, bson, Collection};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document};
use mongodb_net::{Task, Priority };
use serde::{Serialize, Deserialize};
use thiserror::{Error as ThisError};

#[derive(Debug, ThisError)]
pub enum Error {

    #[error("MongoDB Error: {0}")]
    MongoDbError(#[from] mongodb::error::Error),

}

#[derive(Clone)]
pub struct TaskMongoDb {
    client: Client,
    tasks_collection: Collection<Task>
}

impl TaskMongoDb {

    pub async fn connect(url: &str) -> Result<Self, Error> {
        let client = Client::with_uri_str(url).await?;
        let tasks_collection = client.database("task_manager").collection::<Task>("tasks");
        Ok (
            Self {
                client,
                tasks_collection
            }
        )
    }

    pub async fn new_task(&self, title: &str, priority: Priority) -> Result<Task, Error> {
        let task_id = ObjectId::new();
        let task = Task::new(task_id, title, priority, Utc::now().naive_utc());
        self.tasks_collection.insert_one(task.clone()).await?;
        Ok(task)
    }

}
