use thiserror::{Error as ThisError};
use todo_app::{Priority, Task, TaskPgDatabase};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::io::BufReader;
use std::sync::Arc;
use crate::net::Command;

pub mod net;

#[derive(ThisError, Debug)]
enum Error {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("DB error: {0}")]
    DbError(#[from] sqlx::Error),
}

async fn handle_connection(stream: TcpStream, addr: std::net::SocketAddr, db: TaskPgDatabase) -> Result<(), Error> {
    loop {
        let mut length_buffer = [0u8; 4];
        if let Err(_) = stream.read_exact(&mut length_buffer).await {
            println!("Error reading length to buffer, dropping connection.");
            break;
        }
        let len = u32::from_be_bytes(length_buffer) as usize;
        let mut buf = vec![0u8; len];
        if let Err(_) = stream.read_exact(&mut buf).await {
            println!("Unreliable connection, failed to read exact bytes amount in data buffer. Dropping connection.");
            break;
        }
        // Request
        let rq: ClientRequest = bincode::deserialize(&buf)?;
        let mut responses: Arc<Mutex<Vec<CommandResponse>>> = Arc::new(Mutex::new(Vec::with_capacity(capacity)));
        for command in rq.commands {
            tokio::spawn(async move {
                match command {

                    Command::NewTask(title, priority) => {
                        match db.new_task(title, priority).await {
                            Ok(task) => {

                            },
                            Err(e) => {
                                eprintln!("Error creating a new task: {}", e);
                            },
                        }
                    },

                    Command::PendingTasks => {
                        // return the tasks where completed = false
                    },

                    Command::DoneTasks => {
                        // return the tasks where completed == true
                    },

                    Command::MarkTaskDone(id) => {
                        // Mark the task and return a Server Response
                    },

                    Command::EditTaskTitle(task_id, new_title) => {

                    },

                    Command::EditTaskPriority(task_id, priority) => {

                    },

                    Command::QueryTaskById(id) => {

                    },

                });
            }
        }
        
        // Unwrap Vec<CommandResponse>, serialize it and send to client ONLY after all tokio tasks
        // are finished.
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
