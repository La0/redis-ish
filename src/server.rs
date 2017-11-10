use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc,RwLock};
use commands::Command;
use client::{Client, ClientError};
use std::thread;

struct Database {
    lock: RwLock<HashMap<String, String>>,
}

#[derive(Debug)]
enum DBError {
    Locked,
    NoData,
    Exists,
}

impl Database {
    fn new() -> Self {
        Database {
            lock : RwLock::new(HashMap::default()),
        }
    }

    // Read last value using lock
    fn get(& self, key : String) -> Result<String, DBError> {
        match self.lock.read() {
            Ok(data) => data.get(&key).map(|x| x.clone()).ok_or(DBError::NoData),
            Err(_) => Err(DBError::Locked),
        }
    }

    // Put value into database using lock
    // do not allow overriding
    fn put(& self, key : String, value : String) -> Result<String, DBError> {
        match self.lock.write() {
            Ok(mut data) => {
                if data.contains_key(&key) {
                    Err(DBError::Exists)
                } else {
                    data.insert(key.clone(), value);
                    Ok(key)
                }
            },
            Err(_) => Err(DBError::Locked),
        }
    }

    // List all the keys in database
    fn list(&self) -> Result<Vec<String>, DBError> {
        match self.lock.read() {
            Ok(data) => Ok(data.keys().map(|x| x.clone()).collect()),
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
                Ok(Command::List) => {
                    info!("{} >> List", client);
                    match db.list() {
                        Ok(keys) => {
                            client.send(format!("{} keys.", keys.len()));
                            for k in keys {
                                client.send(format!(" > {}", k))
                            }
                        }
                        Err(e) => client.send(format!("Db Error: {:?}", e)),
                    }
                }
                Ok(Command::Get(key)) => {
                    info!("{} >> Get({})", client, key);
                    match db.get(key) {
                        Ok(x) => client.send(format!("Last value = {}", x)),
                        Err(DBError::NoData) => client.send("No data !"),
                        Err(e) => client.send(format!("Db Error: {:?}", e)),
                    }
                }
                Ok(Command::Put(key, value)) => {
                    info!("{} >> Put({}, {})", client, key, value);
                    match db.put(key, value) {
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
