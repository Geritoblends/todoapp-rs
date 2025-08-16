use futures_util::stream::TryStreamExt;
use mongodb::{options::ReturnDocument, Client, Collection};
use mongodb::bson::{oid::ObjectId, doc, to_bson};
use chrono::Utc;
use mongodb_net::{Task, Priority, TaskDocument, DateTimeOutOfRangeError};
use std::str::FromStr;
use thiserror::{Error as ThisError};

#[derive(Debug, ThisError)]
pub enum Error {

    #[error("MongoDB Error: {0}")]
    MongoDbError(#[from] mongodb::error::Error),

    #[error("BSON Serialization error: {0}")]
    BSONSerializationError(#[from] mongodb::bson::ser::Error),

    #[error("ObjectId error: {0}")]
    ObjectIdError(#[from] mongodb::bson::oid::Error),

    #[error("DateTime out of range: {0}")]
    DateOutOfRange(#[from] DateTimeOutOfRangeError),

    #[error("Error: {0}")]
    Custom(String),

}

#[derive(Clone)]
pub struct TaskMongoDb {
    tasks_collection: Collection<TaskDocument>
}

impl TaskMongoDb {

    pub async fn connect(url: &str) -> Result<Self, Error> {
        let client = Client::with_uri_str(url).await?;
        let tasks_collection = client.database("task_manager").collection::<TaskDocument>("tasks");
        Ok (
            Self {
                tasks_collection
            }
        )
    }

    pub async fn new_task(&self, title: &str, priority: Priority) -> Result<Task, Error> {
        let task_id = ObjectId::new().to_hex();
        let task = Task::new(&task_id, title, priority, Utc::now().naive_utc());
        let task_doc = task.as_document()?;
        self.tasks_collection.insert_one(task_doc).await?;
        Ok(task)
    }

    pub async fn pending_tasks(&self) -> Result<Vec<Task>, Error> {
        let filter = doc!{ "completed": false };
        let cursor = self.tasks_collection.find(filter).await?;
        let pending_tasks: Vec<Task> = cursor
            .try_collect::<Vec<TaskDocument>>()
            .await?
            .into_iter()
            .map(|doc| doc.as_task())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pending_tasks)
    }

    pub async fn done_tasks(&self) -> Result<Vec<Task>, Error> {
        let filter = doc!{ "completed": true };
        let cursor = self.tasks_collection.find(filter).await?;
        let completed_tasks: Vec<Task> = cursor
            .try_collect::<Vec<TaskDocument>>()
            .await?
            .into_iter()
            .map(|doc| doc.as_task())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(completed_tasks)
    }

    pub async fn mark_task_done(&self, task_id: &str) -> Result<Task, Error> {
        let oid = ObjectId::from_str(&task_id)?;
        let filter = doc!{ "_id": oid };
        let update = doc!{
            "$set": doc!{ "completed": true }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::After)
            .await?;
        let updated_task: Task = match updated_task {
            Some(task_doc) => task_doc.as_task()?,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };

        Ok(updated_task)
    }

    pub async fn edit_task_title(&self, task_id: &str, title: &str) -> Result<Task, Error> {
        let oid = ObjectId::from_str(&task_id)?;
        let filter = doc!{"_id": oid };
        let update = doc!{
            "$set": doc!{ "title": title }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update).
            return_document(ReturnDocument::After)
            .await?;
        let updated_task: Task = match updated_task {
            Some(task_doc) => task_doc.as_task()?,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };

        Ok(updated_task)
    }

    pub async fn edit_task_priority(&self, task_id: &str, priority: Priority) -> Result<Task, Error> {
        let oid = ObjectId::from_str(&task_id)?;
        let priority = to_bson(&priority)?;
        let filter = doc!{ "_id": oid };
        let update = doc!{
            "$set": doc!{ "priority": priority }
        };
        let updated_task = self.tasks_collection
            .find_one_and_update(filter, update)
            .return_document(ReturnDocument::After)
            .await?;
        let updated_task: Task = match updated_task {
            Some(task_doc) => task_doc.as_task()?,
            None => return Err(Error::Custom("Task not found.".to_string())),
        };
        Ok(updated_task)
    }

    pub async fn query_task_by_id(&self, id: &str) -> Result<Task, Error> {
        let oid = ObjectId::from_str(&id)?;
        let filter = doc!{ "_id": oid};
        if let Some(task_doc) = self.tasks_collection.find_one(filter).await? {
            Ok(task_doc.as_task()?)
        } else {
            Err(Error::Custom("Couldn't find such task.".to_string()))
        }
    }

}
