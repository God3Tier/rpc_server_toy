use std::{io::Read, net::TcpStream};

use crate::Error;

const BUFFER_SIZE: usize = 2 * 1024;
const SPACE_ASCII: u8 = 32;

pub enum Request<'a> {
    Open { path: &'a str, flags: i32 },
    Close { fd: i32 },
    Read { fd: i32, count: usize },
    Write { fd: i32, data: &'a[u8] },

    Lseek { fd: i32, offset: i64, whence: i32 },

    Stat { ver: i32, path:  &'a str },

    Unlink { path:  &'a str },

    GetDirentries { fd: i32, nbytes: usize, basep: i64 },

    GetDirtree { path:  &'a str },
    Default,
    Shutdown,
}

impl <'a> Request<'a>  {
    pub fn is_shutdown(&self) -> bool {
        return matches!(self, Request::Shutdown);
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
                format!("WRITE {} {}", *fd, String::from_utf8((**data).into()).unwrap())
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
            Request::Default => format!("default"),
            Request::Shutdown => format!("shutdown"),
        };
        writeln!(f, "{}\n", command);
        Ok(())
    }
}
