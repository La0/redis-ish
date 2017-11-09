mod server;
mod client;
mod commands;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use server::Server;

fn main() {
    // Setup logger
    pretty_env_logger::init().unwrap();
    info!("Starting Redis-ish");

    // Run server
    let mut server = Server::new(String::from("127.0.0.1:1234"));
    server.run();
}
