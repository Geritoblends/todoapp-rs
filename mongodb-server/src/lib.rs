use futures_util::stream::TryStreamExt;
use mongodb::{options::ReturnDocument, Client, bson, Collection};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Document, to_bson, DateTime};
use mongodb_net::{Task, Priority};
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
        let task_id = ObjectId::new().to_hex();
        let task = Task::new(&task_id, title, priority, DateTime::now());
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

    pub async fn mark_task_done(&self, task_id: &str) -> Result<Task, Error> {
        let filter = doc!{ "_id": task_id };
        let update = doc!{
            "$set": doc!{ "completed": true }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::After)
            .await?;
        let updated_task = match updated_task {
            Some(task) => task,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };

        Ok(updated_task)
    }

    pub async fn edit_task_title(&self, task_id: &str, title: &str) -> Result<Task, Error> {
        let filter = doc!{"_id": task_id };
        let update = doc!{
            "$set": doc!{ "title": title }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update).
            return_document(ReturnDocument::After)
            .await?;
        let updated_task = match updated_task {
            Some(task) => task,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };

        Ok(updated_task)
    }

    pub async fn edit_task_priority(&self, task_id: &str, priority: Priority) -> Result<Task, Error> {
        let priority = to_bson(&priority)?;
        let filter = doc!{ "_id": task_id };
        let update = doc!{
            "$set": doc!{ "priority": priority }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::After)
            .await?;
        let updated_task = match updated_task {
            Some(task) => task,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };
        Ok(updated_task)
    }

    pub async fn query_task_by_id(&self, id: &str) -> Result<Task, Error> {
        let filter = doc!{ "_id": id};
        if let Some(task) = self.tasks_collection.find_one(filter).await? {
            Ok(task)
        } else {
            Err(Error::Custom("Couldn't find such task.".to_string()))
        }
    }

}
