use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use server2::ThreadPool;

enum Reuqest {
    PhysMemory,
    VirtualMemory
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7979").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();


        pool.execute(|| {
            handle_connection(stream);
        });
        

    }
}

fn handle_connection(mut stream: TcpStream) {

    loop {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).unwrap(); // Читаем сырые байты
        let req: String = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        println!("Received: {}", req);
    
    
        // обработка запроса, получеие калла что адо отправить
        match req.trim() {
            "3" => {
                stream.write_all(b"3333").expect("ошибка записи в сокет");
            },
            "4" => {
                stream.write_all(b"4444").unwrap();
            },
            _ => {
                stream.write_all(b"sosat").unwrap();
            }
        }


    
        
    }    
}