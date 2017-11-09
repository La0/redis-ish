use std::net::{TcpListener, TcpStream};
use std::collections::VecDeque;
use commands::Command;
use client::Client;

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

                    // Be nice, say hello
                    client.hello();

                    // Loop on command received
                    loop {
                        match client.wait_command() {

                            Ok(Command::Get) => {
                                //log(stream, String::from("Get"));
                                match self.data.pop_back() {
                                    Some(x) => client.send(String::from(format!("Last value = {}", x))),
                                    None =>	client.send(String::from("No data !")),
                                }
                            }
                            Ok(Command::Put(x)) => {
                                //log(stream, format!("Put({})", x));
                                self.data.push_back(x.clone());
                                client.send(String::from(format!("Stored {}", x)));
                            }
                            Ok(Command::Quit) => {
                                //log(stream, String::from("Quit"));
                                client.quit();
                                break;
                            }
                            Err(e) => {
                                client.send(String::from(format!("Error: {}", e)));
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


fn log(stream: &mut TcpStream, message: String){
	println!("{:?}: {}", stream.peer_addr(), message);
}

