use std::net::{TcpListener, TcpStream, Shutdown};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::str;

#[derive(Debug)]
enum Command {
	Get,
	Put(String),
	Quit,
}

struct Server {
	listener : TcpListener,
	data: VecDeque<String>,
}

impl Server {

	// Create new listening TCP server
	fn new(bind_to: String) -> Server {
		Server {
			listener: TcpListener::bind(bind_to).unwrap(), 
			data: VecDeque::default(),
		}

	}

	fn run(&mut self) {
		loop {

			match self.listener.accept() {
				Ok((stream, _)) => {
					let mut stream = stream;
					self.handle_client(&mut stream);
				}
				Err(e) => println!("couldn't get client: {:?}", e),
			}
		}
	}

	fn handle_client(&mut self, stream: &mut TcpStream) {
		// Be nice, say hello
		log(stream, String::from("New client !"));
		send(stream, String::from("Hello from redis-ish !"));

		loop {
			let mut buffer: [u8; 20] = [0; 20];
			match stream.read(&mut buffer) {
				Ok(size) => {
					log(stream, format!("client sent {} bytes", size));

					if size == 0 { 
						// Avoid looping on closed connection
						break;
					}

					match parse_command(&mut buffer) {

						Ok(Command::Get) => {
							log(stream, String::from("Get"));
							match self.data.pop_back() {
								Some(x) => send(stream, String::from(format!("Last value = {}", x))),
								None =>	send(stream, String::from("No data !")),
							}
						}
						Ok(Command::Put(x)) => {
							log(stream, format!("Put({})", x));
							self.data.push_back(x.clone());
							send(stream, String::from(format!("Stored {}", x)));
						}
						Ok(Command::Quit) => {
							log(stream, String::from("Quit"));
							send(stream, String::from("Bye !"));
							let _ = stream.shutdown(Shutdown::Both);
							break;
						}
						Err(e) => send(stream, String::from(format!("Error: {}", e))),
					}
				}
				Err(e) => {
					println!("No input from client. Err: {}", e);
					break;
				}
			}
		}
	}
}

fn parse_command(buffer: &mut [u8]) -> Result<Command, String> {
	match str::from_utf8(&buffer) {
		Ok(input) => {
			let input = input.trim().to_lowercase(); // remove newline
			match input {
				ref input if input.starts_with("get") => Ok(Command::Get),
				ref input if input.starts_with("put ") => {
					let payload = input[4..].to_string();
					Ok(Command::Put(payload))
				}
				ref input if input.starts_with("quit") => Ok(Command::Quit),
				_ => Err(format!("Invalid command {}", input)),
			}
		}

		Err(e) => Err(format!("Decoder failure : {}", e)),
	}
}

// Helper to send a string on a tcp stream
fn send(stream: &mut TcpStream, message: String) {
	log(stream, format!("Replying > {}", message));
	let message = message + "\n";
	let bytes = message.as_bytes();
	match stream.write(bytes) {
		Ok(_) => (),
		Err(e) => println!("Write failed: {}", e),
	}
}

fn log(stream: &mut TcpStream, message: String){
	println!("{:?}: {}", stream.peer_addr(), message);
}


fn main() {
    println!("Starting Redis-ish");

	let mut server = Server::new(String::from("127.0.0.1:1234"));
	server.run();
}
