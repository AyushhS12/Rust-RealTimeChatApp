use std::{sync::{mpsc, Arc, Mutex}, thread::{self, JoinHandle}};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender:mpsc::Sender<Message>
}

pub struct Worker{
    id:usize,
    thread:Option<JoinHandle<()>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool{
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::new();
        let (sender , receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id,receiver.clone()));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F : FnOnce() + Send + 'static>(&self,f:F) {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap()
    }
}

impl Drop for ThreadPool{
    fn drop(&mut self) {
        for _ in &mut self.workers{
            self.sender.send(Message::Terminate).unwrap()
        }

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker{
    fn new(id:usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                },
                Message::Terminate => {
                    println!("Worker {id} was told to terminate.");
                    break;
                }
            }
        });
        Worker { id: id, thread : Some(thread)}
    }
}