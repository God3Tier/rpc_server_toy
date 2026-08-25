use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

mod dirtreenode;
mod lib;
mod request;
mod threadpool;
use crate::{request::Request, threadpool::Threadpool};
pub type Error = Box<dyn std::error::Error>;

fn main() {
    // println!("Starting server.....");
    let pool = Threadpool::new(4);
    let listener = TcpListener::bind(("0.0.0.0", 15440)).unwrap();
    // println!("Successfully started server");
    for stream in listener.incoming() {
        // println!("Incoming listener");
        // handle_connections(stream);
        pool.execute(move || {
            handle_connections(stream);
        });
    }
}

fn handle_connections(stream: Result<TcpStream, std::io::Error>) {
    if stream.is_err() {
        println!("Unknown stream error {}", stream.err().unwrap());
        return;
    }

    let mut stream = stream.unwrap();

    loop {
        // println!("Generating request");
        let request = Request::new(&mut stream);

        // println!("Checking request");
        if request.is_err() {
        	eprintln!("Request error {:?}", request); 
            // Remove this unwrap
            stream
                .write_all(format!("invalid request {}", request.err().unwrap()).as_bytes())
                .map_err(|err| eprintln!("Unable to write invaid to server {err}"));
            return;
        }
        // println!("Valid request");

        let request = request.unwrap();
        if request.is_shutdown() {
            // println!("Breaking out of loop");
            break;
        }
        // println!("Handling request");
        match request.execute(&mut stream) {
            Err(e) => {
                println!("Unable to write to client {e}");
                stream
                    .write_all(format!("invalid request {}", e).as_bytes())
                    .map_err(|err| println!("Unable to write invalid request to server {err}"));
            }
            Ok(_) => {
                // println!("Completed request");
            }
        }
    }
    // println!("Loop broken");
    stream
        .shutdown(std::net::Shutdown::Both)
        .map_err(|err| eprintln!("Unable to shutdown {err}"));
    // println!("Closed stream");
}
