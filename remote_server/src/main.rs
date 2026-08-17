use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

mod dirtreenode;
mod request;
mod threadpool;
use crate::{request::Request, threadpool::Threadpool};

pub type Error = Box<dyn std::error::Error>;

fn main() {
    let pool = Threadpool::new(4);
    let listener = TcpListener::bind(("127.0.0.1", 8000)).unwrap();
    for stream in listener.incoming() {
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

    let request = Request::new(&mut stream);

    if request.is_err() {
        // Remove this unwrap
        stream
            .write_all(format!("invalid request {}", request.err().unwrap()).as_bytes())
            .unwrap();
        return;
    }

    let request = request.unwrap();

    match request.execute(&mut stream) {
        Err(e) => stream
            .write_all(format!("invalid request {}", e).as_bytes())
            .unwrap(),
        _ => return,
    }
}
