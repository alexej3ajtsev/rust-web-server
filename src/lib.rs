use std::{sync::{Arc, Mutex, mpsc::{self, Sender}}, thread::{self}};

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce() + ?Sized> FnBox for F {
  fn call_box(self: Box<F>) {
    self()
  }
}

type Job = Box<FnOnce() + Send + 'static>;

enum Message {
  NewJob(Job),
  Terminate
}

struct Worker {
  id: i32,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: i32, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || loop {
        let message = receiver.lock().unwrap().recv().unwrap();
        match message {
          Message::NewJob(job) => {
            println!("Worker #{} has started with job", id);
            job();
          },
          Message::Terminate => {
            println!("Worker #{} has terminated", id);
            break;
          }
        }
    });

    Worker {
      id,
      thread: Some(thread)
    }
  }
}

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    for _ in &mut self.workers {
      self.sender.send(Message::Terminate);
    }
    for worker in &mut self.workers {
      println!("Worker#{} is diconnected", worker.id);
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

impl ThreadPool {
  /// # Panics
  /// 
  /// fn new will panic if size (number of threads) will be less or equal to zero 
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);
    let mut workers = Vec::with_capacity(size);
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    for index in 0..size {
      workers.push(Worker::new(index as i32, Arc::clone(&receiver)));
    }

    ThreadPool {
      workers,
      sender,
    }
  }

  pub fn execute<T>(&self, f: T)
    where T: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    self.sender.send(Message::NewJob(job)).unwrap();
  }
}