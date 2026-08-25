use std::{io::Read, net::TcpStream};

mod executer;

use crate::Error;

const BUFFER_SIZE: usize = 2 * 1024;
const SPACE_ASCII: u8 = 32; 

pub enum Request {
    Open { path: String, flags: i32 },
    Close { fd: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, data: Vec<u8> },

    Lseek { fd: i32, offset: i64, whence: i32 },

    Stat { ver: i32, path: String },

    Unlink { path: String },

    GetDirentries { fd: i32, nbytes: usize, basep: i64 },

    GetDirtree { path: String },
    Default,
    Shutdown,
}

impl Request {
    pub fn new(stream: &mut TcpStream) -> Result<Request, Error> {
        let mut raw_bytes = vec![0u8; BUFFER_SIZE];
        // println!("Reading request");
        match stream.read(&mut raw_bytes) {
            Ok(0) => {
                println!("Shutdown from socket read");
                Ok(Request::Shutdown)
            }
            Ok(n) => {
                let value_string = String::from_utf8(raw_bytes[0..n].into()).unwrap();
                println!("Value string: {value_string}");
                let mut args: Vec<Option<String>> = value_string
                    .split(" ")
                    .map(|a| Some(a.to_string()))
                    .collect();
                if args[0].is_none() {
                    return Ok(Request::Default);
                }
                let opcode = args[0].take().unwrap().to_ascii_uppercase();

                if args.len() <= 1 {
                    return Err("Insufficient arguments".into());
                }

                match opcode.as_str() {
                    "OPEN" => {
                        let path = args[1].take().unwrap();

                        if args.len() < 2 {
                            return Ok(Request::Open { path, flags: 0 });
                        }

                        let flag = args[2].take().unwrap().parse();
                        if flag.is_err() {
                            return Err("Invalid flag value".into());
                        }
                        Ok(Request::Open {
                            path,
                            flags: flag.unwrap(),
                        })
                    }
                    "CLOSE" => {
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("invalid file descriptor".into());
                        }
                        Ok(Request::Close { fd: fd.unwrap() })
                    }
                    "READ" => {
                        if args.len() < 2 {
                            return Err("Not enough arguments".into());
                        }
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("invalid file descriptor".into());
                        }

                        let count = args[2].take().unwrap().parse();

                        if count.is_err() {
                            return Err("invalid count".into());
                        }

                        Ok(Request::Read {
                            fd: fd.unwrap(),
                            count: count.unwrap(),
                        })
                    }
                    "WRITE" => {
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        let mut data = Vec::new();
                        for i in 2..args.len() {
                            data.extend_from_slice(args[i].take().unwrap().as_bytes());
                            data.push(SPACE_ASCII); 
                        }

                        Ok(Request::Write {
                            fd: fd.unwrap(),
                            data,
                        })
                    }

                    "LSEEK" => {
                        if args.len() < 3 {
                            return Err("Not enough arguments".into());
                        }
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        let offset = args[2].take().unwrap().parse();

                        if offset.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        let whence = args[3].take().unwrap().parse();

                        Ok(Request::Lseek {
                            fd: fd.unwrap(),
                            offset: offset.unwrap(),
                            whence: whence.unwrap(),
                        })
                    }
                    "STAT" => {
                        if args.len() < 2 {
                            return Err("Not enough arguments".into());
                        }

                        let ver = args[1].take().unwrap().parse();
                        if ver.is_err() {
                            return Err("Version not found".into());
                        }

                        let path = args[2].take().unwrap();
                        Ok(Request::Stat {
                            ver: ver.unwrap(),
                            path,
                        })
                    }
                    "UNLINK" => Ok(Request::Unlink {
                        path: args[1].take().unwrap(),
                    }),
                    "GETDIRENTRIES" => {
                        if args.len() < 4 {
                            return Err("Not enough arguments".into());
                        }
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        let nbytes = args[2].take().unwrap().parse();

                        if nbytes.is_err() {
                            return Err("Invalid file descriptor".into());
                        }
                        let basep = args[3].take().unwrap().parse();

                        if basep.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        Ok(Request::GetDirentries {
                            fd: fd.unwrap(),
                            nbytes: nbytes.unwrap(),
                            basep: basep.unwrap(),
                        })
                    }
                    "GETDIRTREE" => Ok(Request::GetDirtree {
                        path: args[1].take().unwrap(),
                    }),
                    _ => Ok(Request::Default),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn execute(&self, stream: &mut TcpStream) -> Result<(), Error> {
        match self {
            Request::Open { path, flags } => executer::handle_open(path, *flags, stream),
            Request::Close { fd } => executer::handle_close(*fd, stream),
            Request::Read { fd, count } => executer::handle_read(*fd, *count, stream),
            Request::Write { fd, data } => executer::handle_write(*fd, data, stream),
            Request::Lseek { fd, offset, whence } => {
                executer::handle_lseek(*fd, *offset, *whence, stream)
            }
            Request::Stat { ver, path } => executer::handle_stat(*ver, path, stream),
            Request::Unlink { path } => executer::handle_unlink(path, stream),
            Request::GetDirentries { fd, nbytes, basep } => {
                executer::handle_getdirentries(*fd, *nbytes, *basep, stream)
            }
            Request::GetDirtree { path } => executer::handle_getdirtree(path, stream),
            Request::Default => executer::handle_default(stream),
            Request::Shutdown => Err("Never meant to be called".into()),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        return matches!(self, Request::Shutdown);
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match self {
            Request::Open { path, flags } => "open",
            Request::Close { fd } => "close",
            Request::Read { fd, count } => "read",
            Request::Write { fd, data } => "write",
            Request::Lseek { fd, offset, whence } => "lseek",
            Request::Stat { ver, path } => "stat",
            Request::Unlink { path } => "unlink",
            Request::GetDirentries { fd, nbytes, basep } => "getdirentries",
            Request::GetDirtree { path } => "getdirtree",
            Request::Default => "default",
            Request::Shutdown => "shutdown",
        };
        writeln!(f, "{}\n", command);
        Ok(())
    }
}
