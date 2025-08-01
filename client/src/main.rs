use std::net::TcpStream;
use thiserror::Error as ThisError;
use net::*;
use std::io::{Write, Read, stdin};


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
    stdin().read_line(&mut buffer)?;
    let result: u8 = buffer.trim().parse()?;
    Ok(result)
}

fn request_to_server(stream: &mut TcpStream, rq: ClientRequest) -> Result<ServerResponse, Error> {
    let rq_bytes = bincode::serialize(&rq)?;
    let rq_len = rq_bytes.len() as u32;
    stream.write_all(&rq_len.to_be_bytes())?;
    stream.write_all(&rq_bytes[..])?;
    let mut rs_len_buf = [0u8; 4];
    stream.read_exact(&mut rs_len_buf).expect("Connection lost. Shutting down.");
    let rs_len = u32::from_be_bytes(rs_len_buf) as usize;
    let mut rs_buf = vec![0u8; rs_len];
    stream.read_exact(&mut rs_buf).expect("Connection lost. Shutting down.");
    let rs: ServerResponse = bincode::deserialize(&rs_buf)?;
    Ok(rs)
}

fn handle_response(rs: ServerResponse) -> Result<(), Error> {
    let cmd_responses = rs.unwrap();

    for cmd in cmd_responses {
        match cmd {
            CommandResponse::Success(cmd_val) => {
                match cmd_val {
                    CommandResponseValue::NewTask(_task) => {
                        eprintln!("Succesfully created task.");
                    },
                    CommandResponseValue::PendingTasks(tasks) => {
                        for task in tasks {
                            println!("{}", task.format());
                        }
                    },
                    CommandResponseValue::DoneTasks(tasks) => {
                        for task in tasks {
                            println!("{}", task.format());
                        }
                    },
                    CommandResponseValue::MarkTaskDone => {
                        println!("Successfully marked task as done");
                    },
                    CommandResponseValue::EditTaskTitle => {
                        println!("Succesfully changed title")
                    },
                    CommandResponseValue::EditTaskPriority => {
                        println!("Successfully changed priority");
                    },
                    CommandResponseValue::QueryTaskById(task) => {
                        println!("{}", task.format());
                    },
                }
            },
            CommandResponse::Error(e) => println!("Server-side error: {}",e),
        }
    }
    Ok(())
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
                    let rq = ClientRequest::new(&[Command::PendingTasks]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                2 => {
                    let rq = ClientRequest::new(&[Command::DoneTasks]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                3 => {
                    let mut id = String::new();
                    println!("Enter the id:");
                    if let Err(e) = stdin().read_line(&mut id) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    }
                    let id: i32 = match id.trim().parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Error parsing input: {}. Try again.", e);
                            continue;
                        },
                    };
                        
                    let rq = ClientRequest::new(&[Command::QueryTaskById(id)]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                4 => {
                    // new task
                    let mut title = String::new();
                    println!("Enter the task title:");
                    if let Err(e) = stdin().read_line(&mut title) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    };
                    let title = title.trim().to_string();

                    let mut priority = String::new();
                    println!("Introduce the task priority (1: Low, 2: Regular, 3: Urgent):");
                    if let Err(e) = stdin().read_line(&mut priority) {
                        eprintln!("Error reading line: {}. Try again. Yeah, from start sucker.", e);
                        continue;
                    };
                    let priority = match priority.trim().parse() {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("Error parsing: {}. Try again.", e);
                            continue;
                        },
                    };
                    let priority_val: Priority;
                    match priority {
                        1 => priority_val = Priority::Low,
                        2 => priority_val = Priority::Regular,
                        3 => priority_val = Priority::Urgent,
                        _ => {
                            println!("You selected an invalid number, sucker! Try again.");
                            continue;
                        }
                    }
                    let rq = ClientRequest::new(&[Command::NewTask{title, priority: priority_val}]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };


                },
                5 => {
                    // mark as completed
                    let mut id = String::new();
                    println!("Enter the completed task id:");
                    if let Err(e) = stdin().read_line(&mut id) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    }
                    let id: i32 = match id.trim().parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Error parsing input: {}. Try again.", e);
                            continue;
                        },
                    };
                        
                    let rq = ClientRequest::new(&[Command::MarkTaskDone(id)]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                6 => {
                    // edit task title
                    let mut id = String::new();
                    println!("Enter the task id to update:");
                    if let Err(e) = stdin().read_line(&mut id) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    };
                    let id: i32 = match id.trim().parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Error parsing input: {}. Try again.", e);
                            continue;
                        },
                    };

                    let mut title = String::new();
                    println!("Enter the new title:");
                    if let Err(e) = stdin().read_line(&mut title) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    };
                    let title = title.trim().to_string();
                        
                    let rq = ClientRequest::new(&[Command::EditTaskTitle{task_id: id, new_title: title}]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                7 => {
                    // edit task priority
                    let mut id = String::new();
                    println!("Enter the task id to update:");
                    if let Err(e) = stdin().read_line(&mut id) {
                        eprintln!("Error reading line: {}. Try again.", e);
                        continue;
                    };
                    let id: i32 = match id.trim().parse() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("Error parsing input: {}. Try again.", e);
                            continue;
                        },
                    };

                    let mut priority = String::new();
                    println!("Introduce the task priority (1: Low, 2: Regular, 3: Urgent):");
                    if let Err(e) = stdin().read_line(&mut priority) {
                        eprintln!("Error reading line: {}. Try again. Yeah, from start sucker.", e);
                        continue;
                    };
                    let priority = match priority.trim().parse() {
                        Ok(val) => val,
                        Err(e) => {
                            eprintln!("Error parsing: {}. Try again.", e);
                            continue;
                        },
                    };
                    let priority_val: Priority;
                    match priority {
                        1 => priority_val = Priority::Low,
                        2 => priority_val = Priority::Regular,
                        3 => priority_val = Priority::Urgent,
                        _ => {
                            println!("You selected an invalid number, sucker! Try again.");
                            continue;
                        }
                    }
                        
                    let rq = ClientRequest::new(&[Command::EditTaskPriority{task_id: id, priority: priority_val}]);
                    let response = match request_to_server(&mut stream, rq) {
                        Ok(rq) => rq,
                        Err(e) => {
                            eprintln!("Error: {}. Try again", e);
                            continue;
                        }
                    };
                    if let Err(e) = handle_response(response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

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
