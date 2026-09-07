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
 *
 * Current Issue
 * -> I am cloning the entire string to create a write request. I do not know enough about lifetimes to be able to mitigate this. My best guess
 * 	 is I can wrap it up in a pointer to send to the sclient request function but that is practically it.
 */
use crate::{Error, client, request::Request};
use std::time::{Duration, SystemTime};

const O_RDONLY: i32 = 0;
const O_WRITELY: i32 = 1;
// const O_RDWR: i32 = 2;

#[derive(Eq, PartialEq)]
pub struct Entry {
    pub file_name: String,
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

    pub fn read_data(&self, read_count: u32) -> Option<Vec<u8>> {
        if read_count <= self.read_count {
            // NOTE THIS CLONES THE VALUE BE CAREFUL
            Some(self.data[0..read_count as usize].into())
        } else {
            None
        }
        // NOTE THIS CLONES THE VALUE BE CAREFUL
    }

    pub async fn fetch_read(&mut self, read_count: u32) -> Result<Vec<u8>, Error> {
        if self.dirty_bit {
            // Here, I will overwrite all present data (dont care whether append that one is too complex alr)
            self.flush_dirty_bit().await?;
            self.dirty_bit = false;
        }
        let open = open_server_request(&self.file_name, O_RDONLY).await;
        if let Ok(fd) = open {
            let read_request = Request::Read {
                fd,
                count: read_count as usize,
            };

            let response = client::request_from_server(read_request).await;
            if response.is_err() {
                return Err(response.err().unwrap());
            }

            let response = response.unwrap();
            self.read_count = response.len() as u32;
            self.data = response[0..response.len() - 1].into();

            #[allow(unused)]
            close_server_request(fd)
                .await
                .map_err(|err| eprintln!("Server leaking!!! Unable to cloase because {}", err));
            self.last_requested = SystemTime::now();

            // NOTE: EXPENSIVE CLONE HERE
            return Ok(self.data.clone());
        }

        Err(open.err().unwrap())
    }

    pub async fn flush_dirty_bit(&mut self) -> Result<(), Error> {
        let open = open_server_request(&self.file_name, O_WRITELY).await;
        if let Ok(fd) = open {
            let write_request = Request::Write {
                fd,
                data: self.data.clone(),
            };

            let response = client::request_from_server(write_request).await;
            if response.is_err() {
                return Err(response.err().unwrap());
            }
            let response = {
                let response = response.unwrap();
                String::from_utf8(response[0..response.len() - 1].into())
            };

            if response.is_err() {
                return Err(format!("Unable to cast to String {}", response.err().unwrap()).into());
            }
            let response = response.unwrap().parse();
            if response.is_err() {
                return Err(format!("Unable to conver {}", response.err().unwrap()).into());
            }

            let response: i32 = response.unwrap();

            if response < -1 {
                return Err("Unable to flush dirty cash".into());
            }
            #[allow(unused)]
            close_server_request(fd)
                .await
                .map_err(|err| eprintln!("Server leaking!!! Unable to cloase because {}", err));
            self.dirty_bit = false;
        }
        Ok(())
    }

    pub fn is_stale(&self) -> bool {
        let time_difference = self.last_requested.elapsed();

        // At this point, we dont know how to recover the error so best just to drop the entry
        if time_difference.is_err() {
            return true;
        }

        let time_difference = time_difference.unwrap();

        time_difference >= Duration::from_hours(1)
    }
}

async fn close_server_request(fd: i32) -> Result<i32, Error> {
    let close_request = Request::Close { fd };

    let response = client::request_from_server(close_request).await;

    if response.is_err() {
        return Err(format!("Failed to read value {}", response.err().unwrap()).into());
    }

    let response = {
        let response = response.unwrap();
        String::from_utf8(response[0..response.len() - 1].into())
    };

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

async fn open_server_request(file_name: &str, flags: i32) -> Result<i32, Error> {
    let open_request = Request::Open {
        path: file_name,
        flags,
    };

    let response = client::request_from_server(open_request).await;

    if response.is_err() {
        return Err(format!("Failed to read value {}", response.err().unwrap()).into());
    }

    let response = {
        let response = response.unwrap();
        String::from_utf8(response[0..response.len() - 1].into())
    };

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
