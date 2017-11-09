use std::net::{TcpListener, TcpStream, Shutdown};
use std::collections::VecDeque;
use std::io::{Read, Write};
use commands::{Command, parse_command};

pub struct Server {
	listener : TcpListener,
	data: VecDeque<String>,
}

impl Server {

	// Create new listening TCP server
	pub fn new(bind_to: String) -> Server {
		Server {
			listener: TcpListener::bind(bind_to).unwrap(), 
			data: VecDeque::default(),
		}
	}

	pub fn run(&mut self) {
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

