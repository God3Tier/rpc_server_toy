use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    net::TcpStream,
    os::unix::fs::OpenOptionsExt,
    sync::{Arc, RwLock},
};

use libc::{close, lseek, open, read, stat, unlink, write};

use libc::{c_char, c_void, mode_t, off_t};

use crate::Error;

pub fn handle_open(path: &str, flags: i32, stream: &mut TcpStream) -> Result<(), Error> {
    let path = std::ffi::CString::new(path).unwrap();

    let fd = unsafe { libc::open(path.as_ptr() as *const c_char, flags) };

    match stream.write_all(fd.to_string().as_bytes()) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Unable to send msg, {}", err).into()),
    }
}

pub fn handle_close(fd: i32, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("close".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_read(fd: i32, count: usize, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("read".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_write(fd: i32, data: &Vec<u8>, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("write".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_lseek(
    fd: i32,
    offset: i64,
    whence: i32,
    stream: &mut TcpStream,
) -> Result<(), Error> {
    match stream.write_all("lseek".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_stat(ver: i32, path: &String, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("stat".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_getdirentries(
    fd: i32,
    nybtes: usize,
    basep: i64,
    stream: &mut TcpStream,
) -> Result<(), Error> {
    match stream.write_all("getdirentries".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_getdirtree(path: &String, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("getdirtree".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_default(stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("Unkown ".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_unlink(var: i32, path: &String, stream: &mut TcpStream) -> Result<(), Error> {
    match stream.write_all("unlink".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}
