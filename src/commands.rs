use std::str;

#[derive(Debug)]
pub enum Command {
	Get,
	Put(String),
	Quit,
}

pub fn parse_command(buffer: &mut [u8]) -> Result<Command, String> {
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
