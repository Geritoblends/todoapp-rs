use thiserror::Error;
use sqlx::{Row, PgPool, Error as DbError, FromRow};
use chrono::NaiveDateTime;

#[derive(Debug, Error)]
pub enum ErrorType {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Sqlx error: {0}")]
    SqlxError(DbError),
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "priority", rename_all = "PascalCase")]
pub enum Priority {
    Low,
    Regular,
    Urgent,
}

#[derive(FromRow)]
pub struct Task {
    title: String,
    priority: Priority,
    completed: bool,
    id: u64,
    created_at: NaiveDateTime,
}

impl Task {

    fn format(&self) -> String {
        format!("[{?}]: {}", self.priority, self.title)
    }

}

pub struct TaskPgDatabase {
    pool: PgPool
}

impl TaskPgDatabase {

    pub async fn connect(url: &str) -> Result<Self, DbError> {
        let mut pool = PgPool::connect(&url).await?;
        Ok(Self{pool})
    }

    pub async fn new_task(&self, title: String, priority: Priority) -> Result<Task, DbError> {
        let task = sqlx::query_as!(Task,
            r#"
            INSERT INTO tasks (title, priority)
            VALUES ($1, $2)
            RETURNING *;
            "#,
            title,
            priority)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }

    pub async fn pending_tasks(&self) -> Result<Vec<Task>, DbError> {
        let pending_tasks = sqlx::query_as!(Task,
            r#"
            SELECT * FROM tasks
            WHERE completed = $1;
            "#,
            false)
            .fetch_all(&self.pool)
            .await?;
        Ok(pending_tasks)
    }

    pub async fn done_tasks(&self) -> Result<Vec<Task>, DbError> {
        let done_tasks = sqlx::query_as!(Task,
            r#"
            SELECT * FROM tasks
            WHERE completed = $1;
            "#,
            true)
            .fetch_all(&self.pool)
            .await?;
        Ok(done_tasks)
    }

    pub async fn mark_task_done(&self, task_id: u64) -> Result<(), DbError> {
        sqlx::query!(r#"
        UPDATE tasks
        SET completed = $1
        WHERE id = $2;
        "#,
        true,
        task_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn edit_task_title(&self, task_id: u64, title: &str) -> Result<(), DbError> {
        sqlx::query!(r#"
        UPDATE tasks
        SET title = $1
        WHERE id = $2;
        "#,
        title,
        task_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn edit_task_priority(&self, task_id: Uuid, priority: Priority) -> Result<(), DbError> {
        sqlx::query!(r#"
        UPDATE tasks
        SET priority = $1
        WHERE id = $2
        "#,
        priority,
        task_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

}
