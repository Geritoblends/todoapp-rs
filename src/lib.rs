use chrono::NaiveDateTime;
use sqlx::{PgPool, Error as DbError, FromRow, Type};
use serde::{Serialize, Deserialize};

#[derive(Type, Debug, Clone, Copy, Serialize, Deserialize)]
#[sqlx(type_name = "priority")]
#[sqlx(rename_all = "PascalCase")]
pub enum Priority {
    Low,
    Regular,
    Urgent,
}

#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct Task {
    title: String,
    priority: Priority,
    completed: bool,
    id: i32,
    created_at: NaiveDateTime,
}

impl Task {

    pub fn format(&self) -> String {
        format!("[{:?}]: {}", self.priority, self.title)
    }

    pub fn get_title(&self) -> String {
        format!("{}", self.title)
    }

    pub fn get_priority(&self) -> Priority {
        self.priority
    }   

    pub fn get_creation_time(&self) -> NaiveDateTime {
        self.created_at
    }

    pub fn get_status(&self) -> bool {
        self.completed
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

}

#[derive(Clone)]
pub struct TaskPgDatabase {
    pool: PgPool
}

impl TaskPgDatabase {

    pub async fn connect(url: &str) -> Result<Self, DbError> {
        let pool = PgPool::connect(&url).await?;
        Ok(Self{pool})
    }

    pub async fn new_task(&self, title: &str, priority: Priority) -> Result<Task, DbError> {
        let task = sqlx::query_as!(Task,
            r#"
            INSERT INTO tasks (title, priority)
            VALUES ($1, $2)
            RETURNING id, title, priority as "priority: Priority", completed, created_at;
            "#,
            title,
            priority as Priority)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }

    pub async fn pending_tasks(&self) -> Result<Vec<Task>, DbError> {
        let pending_tasks = sqlx::query_as!(Task,
            r#"
            SELECT id, title, priority AS "priority: Priority", completed, created_at FROM tasks
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
            SELECT id, title, priority AS "priority: Priority", completed, created_at FROM tasks
            WHERE completed = $1;
            "#,
            true)
            .fetch_all(&self.pool)
            .await?;
        Ok(done_tasks)
    }

    pub async fn mark_task_done(&self, task_id: i32) -> Result<(), DbError> {
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

    pub async fn edit_task_title(&self, task_id: i32, title: &str) -> Result<(), DbError> {
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

    pub async fn edit_task_priority(&self, task_id: i32, priority: Priority) -> Result<(), DbError> {
        sqlx::query!(r#"
        UPDATE tasks
        SET priority = $1
        WHERE id = $2
        "#,
        priority as Priority,
        task_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn query_task_by_id(&self, task_id: i32) -> Result<Task, DbError> {
        let task = sqlx::query_as!(Task, r#"
        SELECT id, created_at, title, completed, priority AS "priority: Priority" FROM tasks
        WHERE id = $1;"#,
        task_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }

}
