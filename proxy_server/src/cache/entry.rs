/*
 * Entry is a struct that caches any fd commands.
 * It contains
 *  - file name (for server operations)
 *	- Last Requested
 *	- isWritable
 *	- Dirty Bit (To check whether to re write back into file when it is inevitably evicted)
 *	- Count Read
 *	- Data within said counts in [u8] or Vec<u8> (Havent Decided)
 *
 * Design choices
 * - I have delibretely left string path based request out of the equation as that one needs
 * 	 completely different structure and is more of a snapshot that can be changed at any time
 * - For Reading, I will design it such that if it requires increase size, it will directly request
 * 	 from the server
 * - If write is requested, the server will have a pair of the stream and the open allowance. This
 * 	 ensures that there SHOULD be no invalid permissions breaking flow. Write still needs to be designed
 * 	 properly
 */
use crate::{Error, client, request::Request};
use std::time::SystemTime;

const O_RDONLY: i32 = 0;
const O_WRITELY: i32 = 1;
const O_RDWR: i32 = 2;

#[derive(Eq, PartialEq)]
pub struct Entry {
    file_name: String,
    last_requested: SystemTime,
    dirty_bit: bool,
    read_count: u32,
    data: Vec<u8>,
}

impl Entry {
    pub fn new(file_name: String) -> Entry {
        Entry {
            file_name,
            last_requested: SystemTime::now(),
            dirty_bit: false,
            read_count: 0,
            data: Vec::new(),
        }
    }
    pub fn write_data(&mut self, data: Vec<u8>, append: bool) {
        self.dirty_bit = true;
        if append {
            self.read_count += data.len() as u32;
            self.data.extend_from_slice(&data);
        } else {
            self.read_count = data.len() as u32;
            self.data = data;
        }
    }

    pub fn read_data(&mut self, read_count: u32) -> Option<&[u8]> {
        if read_count <= self.read_count {
            Some(&self.data[0..read_count as usize])
        } else {
            None
        }
    }

    pub fn fetch_read(&mut self, read_count: u32, fd: i32) -> Result<&[u8], Error> {
        if self.dirty_bit {
            // Here, I will overwrite all present data (dont care whether append that one is too complex alr)
            let open = open_server_request(&self.file_name, O_WRITELY);
            if let Ok(fd) = open {
                let write_request = Request::Write {
                    fd,
                    data: &self.data,
                };

                let response = client::request_from_server(write_request);
                if response.is_err() {
                    return Err(response.err().unwrap());
                }
                let response = String::from_utf8(response.unwrap());

                if response.is_err() {
                	return Err(format!("Unable to cast to String {}", response.err().unwrap()).into())
                }
                let response = response.unwrap().parse();
                if response.is_err() {
                	return Err(format!("Unable to conver {}", response.err().unwrap()).into())
                }

                let response: i32 = response.unwrap();
                
                if response < -1 {
                	return Err("Unable to flush dirty cash".into());
                }
                
                close_server_request(fd)
                    .map_err(|err| eprintln!("Server leaking!!! Unable to cloase because {}", err));
                self.dirty_bit = false; 
            }
        }
        let open = open_server_request(&self.file_name, O_RDONLY);
        if let Ok(fd) = open {
            let read_request = Request::Read {
                fd,
                count: read_count as usize,
            };

            let response = client::request_from_server(read_request);
            if response.is_err() {
                return Err(response.err().unwrap());
            }

            let response = response.unwrap();
            self.read_count = response.len() as u32;
            self.data = response;
            close_server_request(fd)
                .map_err(|err| eprintln!("Server leaking!!! Unable to cloase because {}", err));
            return Ok(&self.data);
        }

        Err(open.err().unwrap())
    }
}

impl Drop for Entry {
    fn drop(&mut self) {
        let open = open_server_request(&self.file_name, O_WRITELY);
        if let Ok(fd) = open {
            let write_request = Request::Write {
                fd,
                data: &self.data,
            };

            client::request_from_server(write_request)
                .map_err(|err| eprintln!("Failed to write anything from server"));
            close_server_request(fd)
                .map_err(|err| eprintln!("Server leaking!!! Unable to cloase because {}", err));
        } else {
            eprintln!("Unable to open server to close")
        }
    }
}

fn close_server_request(fd: i32) -> Result<i32, Error> {
    let close_request = Request::Close { fd };

    let response = client::request_from_server(close_request);

    if response.is_err() {
        return Err(format!("Failed to read value {}", response.err().unwrap()).into());
    }

    let response = String::from_utf8(response.unwrap());

    if response.is_err() {
        return Err(format!("Failed to convert value {}", response.err().unwrap()).into());
    }

    let response = response.unwrap().parse();

    if response.is_err() {
        return Err(format!("Invalid formating {}", response.err().unwrap()).into());
    }
    let response = response.unwrap();
    if response < -1 {
        return Err("Failed to open server file".into());
    }
    Ok(response)
}

fn open_server_request(file_name: &str, flags: i32) -> Result<i32, Error> {
    let open_request = Request::Open {
        path: file_name,
        flags,
    };

    let response = client::request_from_server(open_request);

    if response.is_err() {
        return Err(format!("Failed to read value {}", response.err().unwrap()).into());
    }

    let response = String::from_utf8(response.unwrap());

    if response.is_err() {
        return Err(format!("Failed to convert value {}", response.err().unwrap()).into());
    }

    let response = response.unwrap().parse();

    if response.is_err() {
        return Err(format!("Invalid formating {}", response.err().unwrap()).into());
    }
    let response = response.unwrap();
    if response < -1 {
        return Err("Failed to open server file".into());
    }
    Ok(response)
}