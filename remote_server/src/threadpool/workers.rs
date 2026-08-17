use super::Job;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub struct Workers {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Workers {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {job()}, 

                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });


        Self {
            id, 
            thread: Some(thread)
        }
    }

}
