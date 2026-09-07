/*
 * For each incoming client request, we need to manage which file they hold and what permissions they have requested along side
 *
 * The reason this is cancerous is that the user essentially is the logger and executer. I can do a similar model to the remote
 * server but it will lead to me passing in permission info like a boolean. However, I do note that shutdow
 */

use crate::{cache::Cache, client, request::Request};
use futures::FutureExt;
use std::{collections::HashMap, future::poll_fn, sync::Arc};
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
        path_mapper: Arc<RwLock<HashMap<i32, String>>>,
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
    path_mapper: Arc<RwLock<HashMap<i32, String>>>,
    cache: Arc<Mutex<Cache>>,
) {
    while let Some(message) = rx.recv().await {
        match Request::new(&message) {
            Ok(request) => match request {
                Request::Open { path, flags } => {
                    let mut cache_lock = cache.lock().await;
                    let fd = cache_lock.generate_fd();
                    drop(cache_lock);

                    let mut user_lock = write_user.write().await;
                    user_lock.file_opened.push(File { fd, flags });
                    drop(user_lock);
                    let mut path_mapper_write = path_mapper.write().await;
                    path_mapper_write.insert(fd, path.to_string());
                    drop(path_mapper_write);

                    write_to_server("-1".as_bytes(), Arc::clone(&writer_stream)).await;
                }
                /*
                 * Here, we clone the file name reference. This is because it can get quite problematic if we await accross a lock
                 * I rather take the acceptable clone process than a potential freeze (and not allowing new file entries) because
                 * the hashmap is still locked.
                 */
                Request::Read { fd, count } => {
                    let file_name = {
                        let path_mapper_read = path_mapper.read().await;
                        let res = path_mapper_read.get(&fd).cloned();
                        drop(path_mapper_read);
                        res
                    };

                    if let Some(file_name) = file_name {
                        let mut cache_lock = cache.lock().await;
                        let value = cache_lock.read_file(fd, count as u32, file_name).await;
                        drop(cache_lock);

                        if let Ok(mut data) = value {
                            data.push(b'\n');
                            write_to_server(&data, Arc::clone(&writer_stream)).await;
                        } else {
                            eprintln!("Unable to read because: {}", value.err().unwrap());
                            write_to_server("-1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                        }
                    } else {
                        eprintln!("FD not found");
                    }
                    write_to_server("-1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                }
                /*
                 * Here, we clone the file name reference. This is because it can get quite problematic if we await accross a lock
                 * I rather take the acceptable clone process than a potential freeze (and not allowing new file entries) because
                 * the hashmap is still locked.
                 */
                Request::Write { fd, data } => {
                    let file_name = {
                        let path_mapper_read = path_mapper.read().await;
                        let res = path_mapper_read.get(&fd).cloned();
                        drop(path_mapper_read);
                        res
                    };

                    if let Some(file_name) = file_name {
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
                        cache_lock.write_file(fd, data, appendable, file_name);
                        drop(cache_lock);

                        write_to_server("1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                    } else {
                        write_to_server("-1\n".as_bytes(), Arc::clone(&writer_stream)).await;
                    }
                }
                _ => {
                    let result = client::request_from_server(request).await;
                    if result.is_err() {
                        eprintln!("Unable to write request to main server",)
                    }
                    let result = result.unwrap();
                    write_to_server(&result, Arc::clone(&writer_stream)).await;
                }
            },
            Err(e) => {
                eprintln!("Invalid request: {e}")
            }
        }
    }
}

async fn write_to_server(buffer: &[u8], writer_stream: Arc<Mutex<TcpStream>>) {
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
                    let data = buf.filled();
                    println!("Peeked: {:?}", data);

                    // Now consume the bytes.
                    let mut consume_buf = vec![0u8; n];

                    match stream.read_exact(&mut consume_buf).await {
                        Ok(_) => {
                            let value = String::from_utf8(consume_buf);
                            if value.is_err() {
                                eprintln!("Invalid string error for {}", value.err().unwrap());
                                continue;
                            }

                            if let Err(e) = rx.send(value.unwrap()).await {
                                eprintln!("Unable to send message :{e}")
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
                    println!("Nothing to read");
                }
            }
        }

        sleep(Duration::from_millis(2)).await;
    }

    drop(listener_stream);
}
