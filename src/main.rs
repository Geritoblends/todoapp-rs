use thiserror::{Error as ThisError};
use todo_app::{Priority, Task, TaskPgDatabase};
use tokio::net::{TcpListener, TcpStream};
use std::io::BufReader;

#[derive(ThisError, Debug)]
enum Error {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("DB error: {0}")]
    DbError(#[from] sqlx::Error),
}

async fn handle_connection(stream: TcpStream, addr: std::net::SocketAddr, db: TaskPgDatabase) -> Result<(), Error> {
    let mut buffer = [0u8; 1024];
    
    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 { break; }
        let bytes = &buffer[..n];
        let data = String::from_utf8_lossy(bytes).into_owned();
        let lines: Vec<&str> = data.lines().collect();

        match lines.first() {

            Some(&"new_task") => {
                // Handle new_task
            },

            Some(&"pending_tasks") => {
                // Return pending tasks
            },

            None => continue,

        }
    }
    
}

#[tokio::main]
async fn main() -> Result<(), Error> {

    let db = TaskPgDatabase::connect("postgres://postgres:mysecretpassword@localhost:5432/postgres").await?;
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                let db = db.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, addr, db).await {
                        eprintln!("Connection handler failed: {}", e);
                    }
                });
            },
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down server");
                break;
            }
        }
    }
    Ok(())

}
