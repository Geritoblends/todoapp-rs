use thiserror::{Error as ThisError};
use todo_app::{Priority, Task, TaskPgDatabase};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bincode::{config};
use std::io::BufReader;
use std::sync::Arc;
use crate::net::*;

pub mod net;

#[derive(ThisError, Debug)]
enum Error {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("DB error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("Encode error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),

    #[error("Decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),
}

async fn handle_connection(
    mut stream: TcpStream,
    addr: std::net::SocketAddr,
    db: TaskPgDatabase
) -> Result<(), Error> {
    loop {
        // Read message length
        let mut length_buffer = [0u8; 4];
        if stream.read_exact(&mut length_buffer).await.is_err() {
            println!("Error reading length to buffer, dropping connection.");
            break;
        }
        
        let len = u32::from_be_bytes(length_buffer) as usize;
        let mut buf = vec![0u8; len];
        if stream.read_exact(&mut buf).await.is_err() {
            println!("Unreliable connection, failed to read exact bytes amount in data buffer. Dropping connection.");
            break;
        }

        // Deserialize request
        let (rq, _): (ClientRequest, usize) = bincode::decode_from_slice(&buf, config::standard())?;
        let commands = rq.get_commands();
        let expected_responses_len = commands.len();
        
        // Create channel with enough capacity
        let (tx, mut rx) = mpsc::channel(expected_responses_len);

        // Spawn tasks for each command
        for command in commands {
            let db = db.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                let response = match command {
                    Command::NewTask { title, priority } => {
                        match db.new_task(title, priority).await {
                            Ok(task) => CommandResponse::Success(
                                CommandResponseValue::NewTask(task)
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::PendingTasks => {
                        match db.pending_tasks().await {
                            Ok(tasks) => CommandResponse::Success(
                                CommandResponseValue::PendingTasks(tasks)
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::DoneTasks => {
                        match db.pending_tasks().await {
                            Ok(tasks) => CommandResponse::Success(
                                CommandResponseValue::DoneTasks(tasks)
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::MarkTaskDone(id) => {
                        match db.mark_task_done(id).await {
                            Ok(_) => CommandResponse::Success(
                                CommandResponseValue::MarkTaskDone
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::EditTaskTitle { task_id, new_title } => {
                        match db.edit_task_title(task_id, new_title).await {
                            Ok(_) => CommandResponse::Success(
                                CommandResponseValue::EditTaskTitle
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::EditTaskPriority { task_id, priority } => {
                        match db.edit_task_priority(task_id, priority).await {
                            Ok(_) => CommandResponse::Success(
                                CommandResponseValue::EditTaskPriority
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                    Command::QueryTaskById(id) => {
                        match db.query_task_by_id(id).await {
                            Ok(task) => CommandResponse::Success(
                                CommandResponseValue::QueryTaskById(task)
                            ),
                            Err(e) => CommandResponse::Error(e.to_string()),
                        }
                    },
                };
                
                // Send response through channel
                if let Err(e) = tx.send(response).await {
                    eprintln!("Failed to send response: {}", e);
                }
            });
        }

        // Drop our sender so the receiver knows when to stop
        drop(tx);

        // Collect all responses
        let mut responses = Vec::with_capacity(expected_responses_len);
        while let Some(response) = rx.recv().await {
            responses.push(response);
        }

        // Create ServerResponse and serialize
        let server_response = ServerResponse {
            payload: responses
        };
        
        let serialized: Vec<u8> = bincode::encode_to_vec(&server_response, config::standard()).unwrap();
        let len = serialized.len() as u32;
        
        // Send length prefix followed by serialized data
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;
    }
    
    Ok(())
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
