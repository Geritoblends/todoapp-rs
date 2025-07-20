use sqlx::{Error as DbError};
use todo_app::{Priority, Task, TaskPgDatabase};

#[tokio::main]
async fn main() -> Result<(), DbError> {
    let url = "postgres://postgres:mysecretpassword@localhost:5432/postgres";
    let db = TaskPgDatabase::connect(url).await?;

    let new_task: Task = db.new_task("Hello from Rust nigger", Priority::Urgent).await?;

    println!("{:?}", new_task);

    Ok(())
}
