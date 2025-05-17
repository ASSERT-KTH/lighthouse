// thread_pool.rs

use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_THREAD_POOL: ThreadPool = ThreadPool::new(1, 5);
}

pub struct ThreadPool {
    sender: SyncSender<Box<dyn FnOnce() + Send + 'static>>,
    workers: Vec<WorkerGuard>,
}

struct WorkerGuard {
    thread: Option<thread::JoinHandle<()>>,
}

impl WorkerGuard {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Box<dyn FnOnce() + Send + 'static>>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                match receiver.lock().unwrap().recv() {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} exiting: channel closed.", id);
                        break;
                    }
                }
            }
        });

        WorkerGuard {
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize, capacity: usize) -> Self {
        assert!(size > 0, "线程池大小必须大于0");
        assert!(capacity > 0, "任务队列容量必须大于0");

        let (sender, receiver) = mpsc::sync_channel(capacity);
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(WorkerGuard::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { sender, workers }
    }

    pub fn try_execute<F>(&self, f: F) -> Result<(), TrySendError<Box<dyn FnOnce() + Send + 'static>>>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.try_send(Box::new(f))
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 等待所有工作线程结束
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Worker thread join failed");
            }
        }

        println!("All threads joined. ThreadPool dropped.");
    }
}
