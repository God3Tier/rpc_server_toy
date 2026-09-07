mod cache;
mod client;
mod request;
mod user;

use crate::{cache::Cache, user::User};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock},
    time::{Duration, sleep},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
const SERVER_ADDRESS: &str = "0.0.0.0:15440";

async fn cron_job_sweeper(cache: Arc<Mutex<Cache>>) {
    let mut cache = cache.lock().await;
    cache.sweep_cache().await;
    drop(cache)
}

#[tokio::main]
async fn main() {
    // I want capacity to be modifiable
    println!("Initialising, cache!");
    let capcacity = 10;
    let cache = Arc::new(Mutex::new(Cache::new(capcacity)));
    let sweeper_pointer = Arc::clone(&cache);

    println!("Initialising user storage");
    let mut users_connected = HashMap::new();

    println!("Initialising, server!");
    let connection = TcpListener::bind(SERVER_ADDRESS).await;

    if connection.is_err() {
        panic!("Unable to connect to server")
    }
    let connection = connection.unwrap();

    let mut client_id = 0;
    let fd_to_file_name = Arc::new(RwLock::new(HashMap::new()));

    /*
     * Spawn cron job
     */
    tokio::spawn(async move {
        sleep(Duration::from_hours(2)).await;
        cron_job_sweeper(sweeper_pointer).await;
    });

    /*
     * Core server logic
     */

    tokio::spawn(async move {
        loop {
            match connection.accept().await {
                Ok((stream, sockaddr)) => {
                    let new_user = User::new(
                        client_id,
                        Arc::new(Mutex::new(stream)),
                        Arc::clone(&fd_to_file_name),
                        Arc::clone(&cache),
                    );
                    users_connected.insert(client_id, new_user);
                    client_id += 1;
                    println!("Ip addr {} has successfully connected", sockaddr)
                }
                Err(e) => {
                    eprintln!("Unable to read from stream {e}")
                }
            }
        }
    });

    // TOOD: Add functionality to print / locate activity logs through http connection (why) or what not. Currently need to just 
    // trust printing logs 
}
