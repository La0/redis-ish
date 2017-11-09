use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::fmt;
use commands::{Command, parse_command};


pub struct Client {
    stream : TcpStream,
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client {:?}", self.stream.peer_addr())
    }
}

impl Client {

    pub fn new(stream : TcpStream) -> Self {
        Client {
            stream: stream,
        }
    }

    pub fn hello(&mut self) {
        self.send(String::from("Hello from redis-ish !"));
    }

    pub fn quit(&mut self) {
        self.send(String::from("Bye !"));
        let _ = self.stream.shutdown(Shutdown::Both);
    }

    // Send a string on a tcp stream
    pub fn send(&mut self, message: String) {
        debug!("{} sending {}", self, message);
        let message = message + "\n";
        let bytes = message.as_bytes();
        match self.stream.write(bytes) {
            Ok(_) => (),
            Err(e) => println!("Write failed: {}", e),
        }
    }

    // Wait for a command on the tcp stream
    pub fn wait_command(&mut self) -> Result<Command, String> {

        let mut buffer: [u8; 20] = [0; 20];
        match self.stream.read(&mut buffer) {
            Ok(size) => {
                debug!("client sent {} bytes", size);

                // Avoid looping on closed connection
                if size == 0 {
                    return Err(String::from("No input"));
                }

                // Output the parsed command
                match parse_command(&mut buffer) {
                    Some(cmd) => Ok(cmd),
                    None => Err(String::from("No command found.")),
                }
            }

            Err(e) => Err(format!("No input from client. Err: {}", e)),
        }
    }

}
