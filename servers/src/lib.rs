use std::{fs::{File, OpenOptions}, io::Write, os::windows::fs::OpenOptionsExt, sync::*, thread};
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;


// Многопоточный сервер
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Клиент для подключенния к лог серверу
pub struct LogClient {
    pipe: Option<File>
}

impl LogClient {
    pub fn new(pipe_name: String) -> Self {
        let mut pipe: File;
        let mut time = 20;
        loop {
            if time == 0 {
                println!("Лог сервер не подключен!");
                return Self {pipe: None};
            }
            time -= 1;
            let pipe_name = format!(r"\\.\pipe\{}", pipe_name);
            let pipe_res = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(FILE_FLAG_OVERLAPPED)
                .open(pipe_name);
            match pipe_res {
                Ok(p) => {
                    pipe = p;
                    break; 
                },
                Err(_) => {},
            }
            println!("Ожидается включение лог сервера, осталось {} секунд!", time + 1);
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
        }
        pipe.write_all(b"SERVER IS ON").unwrap();
        println!("Соединение с лог сервером установлено!");
        return Self { pipe: Some(pipe) };
    } 

    pub fn write_log(&mut self, data: &String) {
        println!("{}", data.trim());
        if self.pipe.is_some() {
            let write_res = self.pipe.as_ref().unwrap().write_all(data.as_bytes());
            if write_res.is_err() {
                println!("\nНе удалось записать логи");
                self.pipe = None;
            }
        }
    }
}

impl Clone for LogClient {
    fn clone(&self) -> Self {
        if self.pipe.is_some() {
            Self { pipe: Some(self.pipe.as_ref().unwrap().try_clone().unwrap()) }
        } else {
            Self {pipe: None}
        }
    }
}

