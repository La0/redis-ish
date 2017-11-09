use std::net::TcpListener;
use std::collections::VecDeque;
use commands::Command;
use client::{Client, ClientError};

pub struct Server {
    listener : TcpListener,
    data: VecDeque<String>,
}

impl Server {

    // Create new listening TCP server
    pub fn new(bind_to: String) -> Self {
        Server {
            listener: TcpListener::bind(bind_to).unwrap(), 
            data: VecDeque::default(),
        }
    }

    pub fn run(&mut self) {
        loop {

            match self.listener.accept() {
                Ok((stream, _)) => {
                    let mut client = Client::new(stream);
                    info!("New {}", client);

                    // Be nice, say hello
                    client.hello();

                    // Loop on command received
                    loop {
                        match client.wait_command() {

                            Ok(Command::Get) => {
                                info!("{} >> Get", client);
                                match self.data.pop_back() {
                                    Some(x) => client.send(format!("Last value = {}", x)),
                                    None => client.send("No data !"),
                                }
                            }
                            Ok(Command::Put(x)) => {
                                info!("{} >> Put({})", client, x);
                                self.data.push_back(x.clone());
                                client.send(format!("Stored {}", x));
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
                            Err(ClientError::ReadFailure(_e)) => {
                                error!("{} : Network read failure, force quit.", client);
                                break;
                            }
                        }
                    }
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
}
