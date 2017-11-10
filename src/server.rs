use std::net::{TcpListener, TcpStream};
use std::collections::VecDeque;
use std::sync::{Arc,RwLock};
use commands::Command;
use client::{Client, ClientError};
use std::thread;

struct Database {
    lock: RwLock<VecDeque<String>>,
}

#[derive(Debug)]
enum DBError {
    Locked,
    NoData,
}

impl Database {
    fn new() -> Self {
        Database {
            lock : RwLock::new(VecDeque::default()),
        }
    }

    // Read last value using lock
    fn get(& self) -> Result<String, DBError> {
        match self.lock.write() {
            Ok(mut data) => data.pop_back().ok_or(DBError::NoData),
            Err(_) => Err(DBError::Locked),
        }
    }

    // Put value into database using lock
    fn put(& self, value : String) -> Result<String, DBError> {
        match self.lock.write() {
            Ok(mut data) => {
                data.push_back(value.clone());
                Ok(value)
            },
            Err(_) => Err(DBError::Locked),
        }
    }
}


pub struct Server {
    listener : TcpListener,
    database: Arc<Database>,
}

impl Server {

    // Create new listening TCP server
    pub fn new(bind_to: String) -> Self {
        Server {
            listener: TcpListener::bind(bind_to).unwrap(), 
            database: Arc::new(Database::new()),
        }
    }

    pub fn run(& self) {

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Create new client thread with a pointer to database
                    let thread_db = Arc::clone(&self.database);
                    thread::spawn(move || Server::new_client(stream, thread_db));
                }
                Err(e) => error!("Network error : couldn't get client {:?}", e),
            }
        }
    }

    // Threaded client management
    fn new_client(stream : TcpStream, db : Arc<Database>) {
        let mut client = Client::new(stream);
        info!("New {}", client);

        // Be nice, say hello
        client.hello();

        // Loop on command received
        loop {
            match client.wait_command() {

                Ok(Command::Get) => {
                    info!("{} >> Get", client);
                    match db.get() {
                        Ok(x) => client.send(format!("Last value = {}", x)),
                        Err(DBError::NoData) => client.send("No data !"),
                        Err(e) => client.send(format!("Db Error: {:?}", e)),
                    }
                }
                Ok(Command::Put(x)) => {
                    info!("{} >> Put({})", client, x);
                    match db.put(x) {
                        Ok(x) => client.send(format!("Stored {}", x)),
                        Err(e) => client.send(format!("Db Error: {:?}", e)),
                    }
                }
                Ok(Command::Quit) => {
                    warn!("{} >> Quit", client);
                    client.quit();
                    break;
                }
                Err(ClientError::InvalidCommand) => {
                    // Do not kill connection on invalid command
                    warn!("{} : Invalid command", client);
                    client.send("Invalid command");
                }
                Err(ClientError::NoInput) => {
                    error!("{} : No input, force quit.", client);
                    break;
                }
                Err(ClientError::DecodingFailed) => {
                    error!("{} : Client stream decoding failed", client);
                    break;
                }
                Err(ClientError::ReadFailure(_e)) => {
                    error!("{} : Network read failure, force quit.", client);
                    break;
                }
            }
        }
    }
}
