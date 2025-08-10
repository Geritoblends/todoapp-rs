use futures_util::stream::TryStreamExt;
use chrono::{NaiveDateTime, Utc};
use mongodb::{Client, bson, Collection};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document, to_bson};
use mongodb_net::{Task, Priority };
use serde::{Serialize, Deserialize};
use thiserror::{Error as ThisError};

#[derive(Debug, ThisError)]
pub enum Error {

    #[error("MongoDB Error: {0}")]
    MongoDbError(#[from] mongodb::error::Error),

    #[error("BSON Serialization error: {0}")]
    BSONSerializationError(#[from] mongodb::bson::ser::Error),

    #[error("Error: {0}")]
    Custom(String),

}

#[derive(Clone)]
pub struct TaskMongoDb {
    tasks_collection: Collection<Task>
}

impl TaskMongoDb {

    pub async fn connect(url: &str) -> Result<Self, Error> {
        let client = Client::with_uri_str(url).await?;
        let tasks_collection = client.database("task_manager").collection::<Task>("tasks");
        Ok (
            Self {
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

    pub async fn pending_tasks(&self) -> Result<Vec<Task>, Error> {
        let filter = doc!{ "completed": false };
        let cursor = self.tasks_collection.find(filter).await?;
        let pending_tasks: Vec<Task> = cursor.try_collect().await?;
        Ok(pending_tasks)
    }

    pub async fn done_tasks(&self) -> Result<Vec<Task>, Error> {
        let filter = doc!{ "completed": true };
        let cursor = self.tasks_collection.find(filter).await?;
        let completed_tasks: Vec<Task> = cursor.try_collect().await?;
        Ok(completed_tasks)
    }

    pub async fn mark_task_done(&self, task_id: ObjectId) -> Result<(), Error> {
        let filter = doc!{ "_id": task_id };
        let update = doc!{
            "$set": doc!{ "completed": true }
        };
        self.tasks_collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn edit_task_title(&self, task_id: ObjectId, title: &str) -> Result<(), Error> {
        let filter = doc!{"_id": task_id };
        let update = doc!{
            "$set": doc!{ "title": title }
        };
        self.tasks_collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn edit_task_priority(&self, task_id: ObjectId, priority: Priority) -> Result<(), Error> {
        let priority = to_bson(&priority)?;
        let filter = doc!{ "_id": task_id };
        let update = doc!{
            "$set": doc!{ "priority": priority }
        };
        self.tasks_collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn query_task_by_id(&self, task_id: ObjectId) -> Result<Task, Error> {
        let filter = doc!{ "_id": task_id};
        if let Some(task) = self.tasks_collection.find_one(filter).await? {
            Ok(task)
        } else {
            Err(Error::Custom("Couldn't find such task.".to_string()))
        }
    }

}
