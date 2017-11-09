use std::str;
use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write, Error as IOError};
use std::fmt;
use commands::{Command, Parser};

pub enum ClientError {
    NoInput,
    InvalidCommand,
    ReadFailure(IOError),
    DecodingFailed,
}

pub struct Client {
    stream : TcpStream,
    parser : Parser,
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
            parser: Parser::new(),
        }
    }

    pub fn hello(&mut self) {
        self.send("Hello from redis-ish !");
    }

    pub fn quit(&mut self) {
        self.send("Bye !");
        let _ = self.stream.shutdown(Shutdown::Both);
    }

    // Send a string on a tcp stream
    // Supports str& and String as message
    pub fn send<T: AsRef<str>>(&mut self, message: T) {
        let message = String::from(message.as_ref()) + "\n";
        debug!("{} sending {}", self, message);
        let bytes = message.as_bytes();
        match self.stream.write(bytes) {
            Ok(_) => (),
            Err(e) => println!("Write failed: {}", e),
        }
    }

    // Wait for a command on the tcp stream
    pub fn wait_command(&mut self) -> Result<Command, ClientError> {

        let mut buffer: [u8; 20] = [0; 20];
        match self.stream.read(&mut buffer) {
            Ok(size) => {
                debug!("client sent {} bytes", size);

                // Avoid looping on closed connection
                if size == 0 {
                    return Err(ClientError::NoInput);
                }

                // Output the parsed command
                match str::from_utf8(&buffer) {
                    Ok(payload) => {
                        match self.parser.find_command(payload) {
                            Some(cmd) => Ok(cmd),
                            None => Err(ClientError::InvalidCommand),
                        }
                    }
                    Err(_) => Err(ClientError::DecodingFailed),
                }
            }

            Err(e) => Err(ClientError::ReadFailure(e)),
        }
    }

}
