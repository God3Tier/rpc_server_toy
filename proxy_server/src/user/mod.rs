/*
 * For each incoming client request, we need to manage which file they hold and what permissions they have requested along side
 *
 * The reason this is cancerous is that the user essentially is the logger and executer. I can do a similar model to the remote
 * server but it will lead to me passing in permission info like a boolean. However, I do note that shutdow
 */

use crate::{Error, cache::Cache, client, request::Request};
use futures::FutureExt;
use std::{future::poll_fn, sync::Arc};
use tokio::{
    io::ReadBuf,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex, RwLock, mpsc},
    time::{Duration, sleep},
};

const BUFFER_SIZE: usize = 2 * 1024;
const O_APPEND: i32 = 1024;

struct File {
    fd: i32,
    flags: i32,
}

pub struct User {
    #[allow(unused)]
    client_id: i32,
    file_opened: Vec<File>,
}

impl User {
    pub fn new(
        client_id: i32,
        stream: Arc<Mutex<TcpStream>>,
        path_mapper: Arc<RwLock<Vec<String>>>,
        cache: Arc<Mutex<Cache>>,
    ) -> Arc<RwLock<User>> {
        let listener_stream = Arc::clone(&stream);
        let writer_stream = Arc::clone(&stream);

        let new_user = Arc::new(RwLock::new(User {
            client_id,
            file_opened: Vec::new(),
        }));

        let write_user = Arc::clone(&new_user);

        let (rx, rw) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move { handle_sender(listener_stream, client_id, rx).await });
        tokio::spawn(async move {
            handle_listener(writer_stream, rw, write_user, path_mapper, cache).await
        });

        Arc::clone(&new_user)
    }
}

async fn handle_listener(
    writer_stream: Arc<Mutex<TcpStream>>,
    mut rx: mpsc::Receiver<String>,
    write_user: Arc<RwLock<User>>,
    path_mapper: Arc<RwLock<Vec<String>>>,
    cache: Arc<Mutex<Cache>>,
) {
    while let Some(message) = rx.recv().await {
        println!("Received message {message}");
        match Request::new(&message) {
            Ok(request) => match request {
                Request::Open { path, flags } => {
                    let mut fd = -1;

                    let path_mapper_read = path_mapper.read().await;
                    for (indx, path_present) in path_mapper_read.iter().enumerate() {
                        if path_present == path {
                            fd = indx as i32;
                            break;
                        }
                    }
                    drop(path_mapper_read);

                    if fd == -1 {
                        let mut cache_lock = cache.lock().await;
                        fd = cache_lock.generate_fd();
                        drop(cache_lock);
                    }

                    let mut user_lock = write_user.write().await;
                    user_lock.file_opened.push(File { fd, flags });
                    drop(user_lock);

                    let mut path_mapper_write = path_mapper.write().await;
                    while fd >= path_mapper_write.len() as i32 {
                        path_mapper_write.push(String::from("UNINIT"))
                    }

                    unsafe {
                        let pointer = path_mapper_write.get_unchecked_mut(fd as usize);
                        *pointer = path.to_string()
                    }

                    drop(path_mapper_write);

                    write_to_client(format!("{fd}\n").as_bytes(), Arc::clone(&writer_stream)).await;
                }
                /*
                 * Here, we clone the file name reference. This is because it can get quite problematic if we await accross a lock
                 * I rather take the acceptable clone process than a potential freeze (and not allowing new file entries) because
                 * the hashmap is still locked.
                 */
                Request::Read { fd, count } => {
                    let file_name = {
                        let path_mapper_read = path_mapper.read().await;
                        let res = unsafe { path_mapper_read.get_unchecked(fd as usize) }.clone();
                        drop(path_mapper_read);
                        res
                    };

                    let mut cache_lock = cache.lock().await;
                    let value = cache_lock.read_file(fd, count as u32, file_name).await;
                    drop(cache_lock);

                    if let Ok(data) = value {
                        // data.push(b'\n');
                        // println!("Data to client: {:?}", data);
                        write_to_client(&data, Arc::clone(&writer_stream)).await;
                    } else {
                        eprintln!("Unable to read because: {}", value.err().unwrap());
                        write_to_client("-1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                    }
                }
                /*
                 * Here, we clone the file name reference. This is because it can get quite problematic if we await accross a lock
                 * I rather take the acceptable clone process than a potential freeze (and not allowing new file entries) because
                 * the hashmap is still locked.
                 */
                Request::Write { fd, data } => {
                    let length = data.len();
                    let file_name = {
                        let path_mapper_read = path_mapper.read().await;
                        let res = unsafe { path_mapper_read.get_unchecked(fd as usize) }.clone();
                        drop(path_mapper_read);
                        res
                    };
                    let mut appendable = false;
                    let read_lock_user = write_user.read().await;
                    let files = &read_lock_user.file_opened;
                    for file in files {
                        if fd == file.fd {
                            appendable = file.flags & O_APPEND != 0;
                            break;
                        }
                    }
                    drop(read_lock_user);
                    let mut cache_lock = cache.lock().await;
                    cache_lock.write_file(fd, data, appendable, file_name).await;
                    drop(cache_lock);

                    write_to_client(
                        format!("{}\n", length).as_bytes(),
                        Arc::clone(&writer_stream),
                    )
                    .await;
                }
                Request::Close { fd } => {
                    let read_user_lock = write_user.read().await;
                    let indx_remove = read_user_lock.file_opened.iter().position(|x| x.fd == fd);
                    drop(read_user_lock);

                    let mut success = false;
                    let mut write_user_lock = write_user.write().await;
                    if let Some(indx) = indx_remove {
                        success = true;
                        write_user_lock.file_opened.remove(indx);
                    }
                    drop(write_user_lock);
                    if success {
                        write_to_client("1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                    } else {
                        println!("Returning unable to close for fd:{fd}");
                        write_to_client("-1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                    }
                }
                _ => {
                    println!("Default call");
                    let result = client::request_from_server(request).await;
                    if result.is_err() {
                        eprintln!("Unable to write request to main server",)
                    }
                    let result = result.unwrap();
                    write_to_client(&result, Arc::clone(&writer_stream)).await;
                }
            },
            Err(e) => {
                eprintln!("Invalid request: {e}")
            }
        }
    }
}

async fn write_to_client(buffer: &[u8], writer_stream: Arc<Mutex<TcpStream>>) {
    let mut writer = writer_stream.lock().await;
    if let Err(e) = writer.write_all(buffer).await {
        eprintln!("Error: Unable to write to client {e}")
    }
    drop(writer)
}

async fn handle_sender(
    listener_stream: Arc<Mutex<TcpStream>>,
    client_id: i32,
    rx: mpsc::Sender<String>,
) {
    loop {
        if let Ok(mut stream) = listener_stream.try_lock() {
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            let mut buf = ReadBuf::new(&mut buf);
            match poll_fn(|cx| stream.poll_peek(cx, &mut buf)).now_or_never() {
                Some(Ok(0)) => {
                    println!("Socket has closed for user {client_id}");
                    break;
                }
                Some(Ok(n)) => {
                    // let data = buf.filled();
                    // println!("Peeked: {:?}", data);

                    // Now consume the bytes.
                    let mut consume_buf = vec![0u8; n];
                    // println!("Attempting to read bytes");
                    match stream.read_exact(&mut consume_buf).await {
                        Ok(_) => {
                            // println!("Consuming buffer");
                            let mut size_start = 0;
                            while size_start < n {
                                match request_parser(&consume_buf[size_start..consume_buf.len()]) {
                                    Ok((value, next_command)) => {
                                        println!("Message sending: {}", value);
                                        // println!("Sending message to receiver");																								//
                                        if let Err(e) = rx.send(value).await {
                                            eprintln!("Unable to send message :{e}")
                                        }
                                        size_start += next_command + 1;
                                    }
                                    Err(e) => {
                                        eprintln!("{e}");
                                        break;
                                    }
                                }
                            }

                            // Move and send some sort of message of receiver to other side to handle
                        }
                        Err(e) => {
                            eprintln!("Read error: {e}");
                            break;
                        }
                    }
                }
                Some(Err(e)) => {
                    eprintln!("Err found {e}")
                }
                None => {
                    // println!("Nothing to read");
                }
            }
        }

        sleep(Duration::from_millis(2)).await;
    }

    drop(listener_stream);
}

fn request_parser(buf: &[u8]) -> Result<(String, usize), Error> {
    println!(
        "Consumed: {:?}",
        buf.iter().map(|&x| x as char).collect::<Vec<char>>()
    );

    let indx_of_first_space = buf.iter().position(|&x| x == b' ');

    if indx_of_first_space.is_none() {
        return Err("Invalid command format".into());
    }

    let indx_of_first_space = indx_of_first_space.unwrap();

    let size_raw = String::from_utf8(buf[0..indx_of_first_space].into())?;
    let size: usize = size_raw.parse()?;
    println!("{size}");
    // Uh.....
    let next_command = indx_of_first_space + size;
    let value = String::from_utf8(buf[indx_of_first_space + 1..next_command + 1].into())?;

    Ok((value, next_command))
}
