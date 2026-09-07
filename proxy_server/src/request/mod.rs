use std::{io::Read, net::TcpStream};

use crate::Error;

const SPACE_ASCII: u8 = 32;

pub enum Request<'a> {
    Open { path: &'a str, flags: i32 },
    Close { fd: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, data: Vec<u8> },

    Lseek { fd: i32, offset: i64, whence: i32 },

    Stat { ver: i32, path: &'a str },

    Unlink { path: &'a str },

    GetDirentries { fd: i32, nbytes: usize, basep: i64 },

    GetDirtree { path: &'a str },
    Default,
}

impl<'a> Request<'a> {
    pub fn new(raw: &'a str) -> Result<Request<'a>, Error> {
        let mut args: Vec<Option<&str>> = raw.split(" ").map(Some).collect();
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

                if args.len() < 2 {
                    return Err("Nothing to write".into());
                }

                let mut data = Vec::new();
                data.extend_from_slice(args[2].take().unwrap().as_bytes());
                for i in 3..args.len() {
                    data.push(SPACE_ASCII);
                    data.extend_from_slice(args[i].take().unwrap().as_bytes());
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
}

#[allow(unused)]
impl<'a> std::fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match self {
            Request::Open { path, flags } => format!("OPEN {path} {flags}"),
            Request::Close { fd } => format!("CLOSE {}", *fd),
            Request::Read { fd, count } => format!("READ {} {}", *fd, *count),
            // TODO: Note this clone here is expensive. It really should only be used to send the message back to server
            Request::Write { fd, data } => {
                format!(
                    "WRITE {} {}",
                    *fd,
                    String::from_utf8((**data).into()).unwrap()
                )
            }
            Request::Lseek { fd, offset, whence } => {
                format!("LSEEK {} {} {}", *fd, *offset, *whence)
            }
            Request::Stat { ver, path } => format!("STAT {} {}", *ver, *path),
            Request::Unlink { path } => format!("UNLINK {}", *path),
            Request::GetDirentries { fd, nbytes, basep } => {
                format!("GETDIRENTRIES {} {} {}", *fd, *nbytes, *basep)
            }
            Request::GetDirtree { path } => format!("GETDIRTREE {}", *path),
            Request::Default => "default".to_string(),
        };
        writeln!(f, "{}\n", command);
        Ok(())
    }
}
