use std::{io::Write, net::TcpStream};

use libc::{c_char, c_void};

use crate::{Error, dirtreenode::DirTreeNodes, getdirentries};

pub fn handle_open(path: &str, flags: i32, stream: &mut TcpStream) -> Result<(), Error> {
    println!("Open called");
    let path = std::ffi::CString::new(path).unwrap();

    let fd = unsafe { libc::open(path.as_ptr() as *const c_char, flags) };
    if fd == -1 {
        let err = std::io::Error::last_os_error();
        println!("Result {fd}, errno: {err}");
    } else {
        println!("Result {fd}");
    }
    match stream.write_all(format!("{}\n", fd).as_bytes()) {
        Ok(_) => {
            // println!("Successfully written to client");
            Ok(())
        }
        Err(err) => {
            // eprintln!("Failed to write to router {err}". );
            Err(format!("Unable to send msg, {}", err).into())
        }
    }
}

pub fn handle_close(fd: i32, stream: &mut TcpStream) -> Result<(), Error> {
    println!("Close called");
    let result = unsafe { libc::close(fd) };
    match stream.write_all(format!("{}\n", result).as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_read(fd: i32, count: usize, stream: &mut TcpStream) -> Result<(), Error> {
    println!("Read called");
    let mut buffer = vec![0u8; super::BUFFER_SIZE];
    let ptr = buffer.as_mut_ptr() as *mut libc::c_void;
    let result = unsafe { libc::read(fd, ptr, count) };
    println!("Read result {:?}", String::from_utf8(buffer.clone()[0..count].into())); 
    if result == -1 {
        let err = std::io::Error::last_os_error();
        println!("errno: {err}");
        match stream.write_all(format!("{}\n", result).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    } else {
        match stream.write_all(&buffer[0..count]) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    }
}

pub fn handle_write(fd: i32, data: &[u8], stream: &mut TcpStream) -> Result<(), Error> {
    println!("Write called");
    let result = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len()) };

    if result == -1 {
        let err = std::io::Error::last_os_error();
        println!("Write Result {result}, errno: {err}");
    } else {
        println!("Write Result {result}");
    }

    match stream.write_all(format!("{}\n", result).as_bytes()) {
        Ok(_) => {
            println!("Write command successful");
            Ok(())
        }
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_lseek(
    fd: i32,
    offset: i64,
    whence: i32,
    stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("Lseek called");
    let result = unsafe { libc::lseek(fd, offset, whence) };
    match stream.write_all(format!("{}\n", result).as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_stat(ver: i32, path: &str, stream: &mut TcpStream) -> Result<(), Error> {
    println!("Stat called");
    let mut buffer: std::mem::MaybeUninit<libc::stat> = std::mem::MaybeUninit::zeroed();
    let buf_ptr = buffer.as_mut_ptr();
    let result = unsafe { libc::stat(path.as_ptr() as *const c_char, buf_ptr) };
    if result == -1 {
        match stream.write_all(format!("{}\n", result).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    } else {
        let buf = unsafe {
            format!(
                "{},{},{},{},{},{},{}\n",
                buffer.assume_init().st_dev as u64,
                buffer.assume_init().st_ino,
                buffer.assume_init().st_mode as u32,
                buffer.assume_init().st_nlink as u64,
                buffer.assume_init().st_uid,
                buffer.assume_init().st_size,
                buffer.assume_init().st_mtime,
            )
        };

        match stream.write_all(buf.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    }
}

pub fn handle_unlink(path: &str, stream: &mut TcpStream) -> Result<(), Error> {
    println!("Unlink called");
    let result = unsafe { libc::unlink(path.as_ptr() as *mut c_char) };
    match stream.write_all(format!("{}\n", result).as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}

pub fn handle_getdirentries(
    fd: i32,
    nbytes: usize,
    basep: i64,
    stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("GetDirentries called");
    let mut buffer = vec![0u8; super::BUFFER_SIZE];
    let basep_ptr: *const i64 = &basep as *const i64;
    let result = unsafe {
        getdirentries::getdirentries(
            fd,
            buffer.as_mut_ptr() as *mut c_void,
            nbytes,
            basep_ptr as *mut getdirentries::off_t,
        )
    };
    if result != -1 {
        match stream.write_all(format!("{}\n", String::from_utf8(buffer).unwrap()).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    } else {
        match stream.write_all("-1\n".as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    }
}

pub fn handle_getdirtree(path: &str, stream: &mut TcpStream) -> Result<(), Error> {
    println!("GetDirtree called");
    let tree = DirTreeNodes::getdirtree(path.to_string());
    if tree.is_err() {
        match stream.write_all("-1\n".as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    } else {
        match stream.write_all(format!("{}\n", tree.unwrap()).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Unable to send message {}", e).into()),
        }
    }
}

pub fn handle_default(stream: &mut TcpStream) -> Result<(), Error> {
    println!("Unknown called");
    match stream.write_all("-1\n".as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Unable to send message {}", e).into()),
    }
}
