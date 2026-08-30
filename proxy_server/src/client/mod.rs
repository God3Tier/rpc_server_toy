use std::net::TcpStream;
use std::io::{Read, BufReader, BufWriter, Write};

use crate::{Error, request};

pub fn request_from_server(req: request::Request) -> Result<Vec<u8>, Error>{
	let server_addr = std::env::var("SERVER_ADDRESS");
	if server_addr.is_err() {
		return Err("Server address not set".into()); 
	}

	let stream = TcpStream::connect(server_addr.unwrap()); 

	if stream.is_err() {
		return Err("Unable to connect to real server".into())
	}

	let stream = stream.unwrap(); 

	let mut reader = BufReader::new(stream.try_clone()?);
	let mut writer = BufWriter::new(stream); 

	if writer.write_all(format!("{}\n", req).as_bytes()).is_err() {
		return Err("Failed to write to server".into())
	}

	writer.flush()?;

	let mut response = Vec::new();

	if reader.read_exact(&mut response).is_err() {
		return Err("Failed to read from server".into()); 
	}
	
	Ok(response)
}