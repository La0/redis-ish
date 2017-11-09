use std::str;

#[derive(Debug)]
pub enum Command {
	Get,
	Put(String),
	Quit,
}

pub fn parse_command(buffer: &mut [u8]) -> Option<Command> {
	match str::from_utf8(&buffer) {
		Ok(input) => {
			let input = input.trim().to_lowercase(); // remove newline
			match input {
				ref input if input.starts_with("get") => Some(Command::Get),
				ref input if input.starts_with("put ") => {
					let payload = input[4..].to_string();
					Some(Command::Put(payload))
				}
				ref input if input.starts_with("quit") => Some(Command::Quit),
				_ => None
			}
		}

		Err(e) => {
            error!("Decoder failure : {}", e);
            None
        }
	}
}
