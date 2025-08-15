use std::collections::BTreeMap;
use std::net::TcpStream;
use thiserror::Error as ThisError;
use mongodb_net::{ServerResponse, Task, Priority, ClientRequest, Command, CommandResponse, CommandResponseValue};
use std::io::{Write, Read, stdin};
use mongodb::bson::{DateTime};
use mongodb::bson::oid::ObjectId;

#[derive(ThisError, Debug)]
enum Error {

    #[error("I/O Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("{0}")]
    Custom(String),

}

#[derive(Clone)]
struct TaskLocalStore {
    tasks: BTreeMap<String, Task>
}

impl TaskLocalStore {

    fn new() -> Self {
        let mut tasks: BTreeMap<String, Task> = BTreeMap::new();
        Self {
            tasks,
        }
    }

    fn upsert(&mut self, task: Task) {
        self.tasks.insert(task.get_id(), task);
    }

    fn select_id(&self) -> Result<String, Error> {
        if !self.tasks.is_empty() {
            println!("== fetched tasks list ==");
            let mut i: u8 = 0;
            for (id, task) in self.tasks.iter() {
                println!("{}. {}", i + 1, task.get_title());
                i += 1
            }
            println!("Select a task (1/{}):", i + 1);
            let mut selected = String::new();
            stdin().read_line(&mut selected)?;
            let selected: u8 = selected.trim().parse::<u8>()? - 1;
            if let Some((id, task)) = self.tasks.iter().nth(selected as usize) {
                return Ok(task.get_id());
            }
        }
        Err(Error::Custom("The local store is empty, try fetching some values".to_string()))
    }

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

fn handle_response(mut store: TaskLocalStore, rs: ServerResponse) -> Result<(), Error> {
    let cmd_responses = rs.unwrap();

    for cmd in cmd_responses {
        match cmd {
            CommandResponse::Success(cmd_val) => {
                match cmd_val {
                    CommandResponseValue::NewTask(task) => {
                        store.upsert(task);
                        eprintln!("Succesfully created task.");
                    },
                    CommandResponseValue::PendingTasks(tasks) => {
                        for task in tasks {
                            println!("{}", task.format());
                            store.upsert(task);
                        }
                    },
                    CommandResponseValue::DoneTasks(tasks) => {
                        for task in tasks {
                            println!("{}", task.format());
                            store.upsert(task);
                        }
                    },
                    CommandResponseValue::MarkTaskDone(task) => {
                        store.upsert(task);
                        println!("Successfully marked task as done");
                    },
                    CommandResponseValue::EditTaskTitle(task) => {
                        println!("Succesfully changed title");
                        store.upsert(task);
                    },
                    CommandResponseValue::EditTaskPriority(task) => {
                        println!("Successfully changed priority");
                        store.upsert(task);
                    },
                    CommandResponseValue::QueryTaskById(task) => {
                        println!("{}", task.format());
                        store.upsert(task);
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
    let mut store = TaskLocalStore::new();
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
                    if let Err(e) = handle_response(store.clone(), response) {
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
                    if let Err(e) = handle_response(store.clone(), response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                3 => {
                    // Show the TaskLocalStore available tasks and obtain the selected id
                    let id = match store.select_id() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("{}", e);
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
                    if let Err(e) = handle_response(store.clone(), response) {
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
                    if let Err(e) = handle_response(store.clone(), response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };


                },
                5 => {
                    // mark as completed
                    let id = match store.select_id() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("{}", e);
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
                    if let Err(e) = handle_response(store.clone(), response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                6 => {
                    // edit task title

                    let id = match store.select_id() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("{}", e);
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
                    if let Err(e) = handle_response(store.clone(), response) {
                        eprintln!("Error handling the response: {}, try again.", e);
                        continue;
                    };

                },
                7 => {
                    // edit task priority
                    let id = match store.select_id() {
                        Ok(id) => id,
                        Err(e) => {
                            eprintln!("{}", e);
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
                    if let Err(e) = handle_response(store.clone(), response) {
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
