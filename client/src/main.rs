use std::io::{self, Read, Write};
use std::net::TcpStream;

#[warn(dead_code)]
enum Reuqest {
    PhysMemory,
    VirtualMemory
}


fn main() -> io::Result<()> {
    // Подключаемся к серверу
    let mut is_stream1 = false;
    let mut is_stream2 = false;
    let mut stream1 = TcpStream::connect("127.0.0.1:7878");
    let mut stream2 = TcpStream::connect("127.0.0.1:7979");
    if stream1.is_ok() {
        is_stream1 = true;
    }
    if stream2.is_ok() {
        is_stream2 = true;
    }
    println!("Connected to server!");



    loop {
        print!("\n\n");
        if is_stream1 {
            print!("1 - название используемого видеоадаптера\n");
            print!("2 (100-1000) - скрыть окно сервера на * мс\n");
        }
        if is_stream2 {
            print!("3 - процент используемой физической памяти\n");
            print!("4 - процент используемой виртуальной памяти\n");
        }
    
        // ввод запроса от клиета
        let mut input = String::new();
        io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
        print!("\n\n");

        let args: Vec<&str> = input.split(" ").collect();
        if args.len() < 1 {
            println!("Недостаточно аргументов!");
            continue;
        }
        let arg1 = args.get(0).unwrap();
        let response;
        match arg1.trim() {
            "1" => {
                if is_stream1 {
                    (stream1, response) = send_request(stream1, input).unwrap();
                    println!("Используемый видеоадаптер: {}", response)
                }
            },
            "2" => {
                if is_stream1 {
                    if args.len() < 2 {
                        println!("Недостаточно аргументов");
                        continue;
                    }
                    let time = args
                        .get(1)
                        .unwrap().trim()
                        .parse::<i32>();
                    match time {
                        Ok(_) => {
                            (stream1, response) = send_request(stream1, input).unwrap();
                            println!("{}", response)
                        },
                        Err(_) => {
                            println!("Аргумент должен быть числом!!!");
                            continue;
                        },
                    }
                }
            }
            "3" => {
                if is_stream2 {
                    (stream2, response) = send_request(stream2, input).expect("Ошибка записи в сокет");
                    println!("Процент используемой физической памяти: {}", response);
                }
            }, 
            "4" => {
                if is_stream2 {
                    (stream2, response) = send_request(stream2, input).expect("Ошибка записи в сокет");
                    println!("Процент используемой виртуальной памяти: {}", response)
                }
            }
            _ => {
                
                // stream = send_request(stream, String::from("sdasdasdasd")).unwrap();
            }
        }




        // send_request(stream, input)?;

    }


}


fn send_request(stream_res: Result<TcpStream, io::Error>, request: String) -> io::Result<(Result<TcpStream, io::Error>, String)> {
    let mut stream = stream_res.unwrap();
    stream.write_all(request.as_bytes()).expect("");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    return io::Result::Ok((Result::Ok(stream), response.into_owned()));

}