mod workers;

use self::workers::Workers;
use std::sync::{Arc, Mutex, mpsc};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Threadpool {
    workers: Vec<Workers>,
    sender: Option<mpsc::Sender<Job>>,
}

impl Threadpool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::new();
        for id in 0..size {
            workers.push(Workers::new(id, Arc::clone(&receiver)));
        }

        Threadpool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Worker with id {} is being shutdown", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
        }
    }
}
