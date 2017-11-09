mod server;
mod client;
mod commands;

use server::Server;

fn main() {
    println!("Starting Redis-ish");

	let mut server = Server::new(String::from("127.0.0.1:1234"));
	server.run();
}
