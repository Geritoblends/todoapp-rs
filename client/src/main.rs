use std::net::TcpStream;
use thiserror::Error as ThisError;
use net::*;
use std::io::{Write, Read};


#[derive(ThisError, Debug)]
enum Error {

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

}

fn menu() -> Result<u8, Error> {
    println!("1. Print pending tasks");
    println!("2. Print completed tasks");
    println!("3. Query a task by id");
    println!("4. Create a new task");
    println!("5. Mark a task as completed");
    println!("6. Edit a task title");
    println!("7. Edit a task priority");
    println!("Choose an option (1/7): ");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let result: u8 = buffer.trim().parse()?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut stream = TcpStream::connect("127.0.0.1:8992").expect("Failed to connect to server. Panicking.");
    println!("Connection successful");
    println!("=== Tasks App ===");

    loop {
        let option = menu();
        match option {
            Ok(n) => match n {
                1 => {
                    let cmd_bytes: Vec<u8> = match bincode::serialize(&Command::PendingTasks) {
                        Ok(vec) => vec,
                        Err(e) => {
                            println!("Error serializing: {}. Try again.", e);
                            continue;
                        },
                    };
                    let len = cmd_bytes.len();
                    if let Err(e) = stream.write_all(&len.to_be_bytes()) {
                        println!("Error writing length bytes: {} Try again.", e);
                        continue;
                    };
                    if let Err(e) = stream.write_all(&cmd_bytes[..]) {
                        println!("Error writing payload bytes. Try again.");
                        continue;
                    };

                    // Response
                    let mut len_buf = [0u8; 4];
                    stream.read_exact(&mut len_buf).expect("Connection lost. Shutting down.");
                    let len = u32::from_be_bytes(len_buf) as usize;
                    let mut response_buf: Vec<u8> = Vec::with_capacity(len);
                    stream.read_exact(&mut response_buf).expect("Connection lost. Shutting down.");
                    
                    let response: ServerResponse = match bincode::deserialize(&mut response_buf) {
                        Ok(response) => response,
                        Err(e) => {
                            println!("Error deserializing: {}. Try again.", e);
                            continue;
                        }
                    };



                },
                2 => {
                    let cmd_bytes: Vec<u8> = match bincode::serialize(&Command::DoneTasks) {
                        Ok(vec) => vec,
                        Err(e) => {
                            println!("Error serializing: {}. Try again.", e);
                            continue;
                        },
                    };
                    let len = cmd_bytes.len();
                    if let Err(e) = stream.write_all(&len.to_be_bytes()) {
                        println!("Error writing length bytes: {} Try again.", e);
                        continue;
                    };
                    if let Err(e) = stream.write_all(&cmd_bytes[..]) {
                        println!("Error writing payload bytes. Try again.");
                        continue;
                    };

                },
                3 => {
                    // query by id
                },
                4 => {
                    // new task
                },
                5 => {
                    // mark as completed
                },
                6 => {
                    // edit task title
                },
                7 => {
                    // edit task priority
                },
                _ => {
                    println!("Invalid number, try again.");
                    continue;
                },
            },


            Err(_) => {
                println!("Error with option selection. Try again.");
                continue;
            }
        }
    }
}
