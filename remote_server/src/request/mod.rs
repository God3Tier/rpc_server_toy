use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    net::TcpStream,
    sync::{Arc, RwLock},
};

mod executer;

use crate::Error;

const BUFFER_SIZE: usize = 2 * 1024 * 1024;

pub enum Request {
    Open { path: String, flags: i32 },
    Close { fd: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, data: Vec<u8> },

    Lseek { fd: i32, offset: i64, whence: i32 },

    Stat { ver: i32, path: String },

    Unlink { var: i32, path: String },

    GetDirentries { fd: i32, nbytes: usize, basep: i64 },

    GetDirtree { path: String },
    Default,
}

impl Request {
    pub fn new(stream: &mut TcpStream) -> Result<Request, Error> {
        let mut raw_bytes: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        match stream.read_exact(&mut raw_bytes) {
            Ok(_) => {
                let value_string = String::from_utf8(raw_bytes.into()).unwrap();
                let mut args: Vec<Option<String>> = value_string
                    .split(" ")
                    .map(|a| Some(a.to_string()))
                    .collect();
                if args[0].is_none() {
                    return Ok(Request::Default);
                }
                if args.len() <= 1 {
                    return Err("Insufficient arguments".into());
                }

                match args[0].take().unwrap().to_ascii_uppercase().as_str() {
                    "OPEN" => {
                        let path = args[1].take().unwrap();

                        if args.len() < 2 {
                            return Ok(Request::Open { path, flags: -1 });
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
                            data.extend_from_slice(args[i].take().unwrap().as_bytes())
                        }

                        Ok(Request::Write {
                            fd: fd.unwrap(),
                            data,
                        })
                    }

                    "LSEEK" => {
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
                    "UNLINK" => {
                        let var = args[1].take().unwrap().parse();

                        if var.is_err() {
                            return Err("Invalid var".into());
                        }

                        Ok(Request::Unlink {
                            var: var.unwrap(),
                            path: args[2].take().unwrap(),
                        })
                    }
                    "GETDIRENTRIES" => {
                        let fd = args[1].take().unwrap().parse();

                        if fd.is_err() {
                            return Err("Invalid file descriptor".into());
                        }

                        let nbytes = args[2].take().unwrap().parse();

                        if nbytes.is_err() {
                            return Err("Invalid file descriptor".into());
                        }
                        let basep = args[2].take().unwrap().parse();

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
            Request::Open { path, flags } => {
                return executer::handle_open(path, *flags, stream);
            }
            Request::Close { fd } => return executer::handle_close(*fd, stream),
            Request::Read { fd, count } => {
                return executer::handle_read(*fd, *count, stream);
            }
            Request::Write { fd, data } => {
                return executer::handle_write(*fd, data, stream);
            }
            Request::Lseek { fd, offset, whence } => {
                return executer::handle_lseek(*fd, *offset, *whence, stream);
            }
            Request::Stat { ver, path } => return executer::handle_stat(*ver, path, stream),
            Request::Unlink { var, path } => {
                return executer::handle_unlink(*var, path, stream);
            }
            Request::GetDirentries { fd, nbytes, basep } => {
                return executer::handle_getdirentries(*fd, *nbytes, *basep, stream);
            }
            Request::GetDirtree { path } => {
                return executer::handle_getdirtree(path, stream);
            }
            Request::Default => return executer::handle_default(stream),
            _ => {}
        }

        Ok(())
    }
}
