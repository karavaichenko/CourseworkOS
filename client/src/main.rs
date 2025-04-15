use core::time;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::{thread, sync::*};


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
        print!("q - выход\n");
    
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

        let repeat_flag = "-r";
        let repeat_flag_index = args.iter().position(|&x| x == repeat_flag);
        let mut repeat_time: i32 = 1000;
        if repeat_flag_index.is_some() {
            let repeat_time_res = args.get(repeat_flag_index.unwrap() + 1);
            if repeat_time_res.is_some() {
                let repeat_time_cast_res = repeat_time_res.unwrap().trim().parse::<i32>();
                if repeat_time_cast_res.is_ok() {
                    repeat_time = repeat_time_cast_res.unwrap();
                } else {
                    print!("Время должно быть числом!")
                }
            }
        }

        let response;
        match arg1.trim() {
            "1" => {
                if is_stream1 {
                    if repeat_flag_index.is_some() {
                        stream1 = repeat_request(stream1, input, repeat_time, String::from("Видеоадаптер: "));
                    } else {
                        (stream1, response) = send_request(stream1, &input).unwrap();
                        println!("Видеоадаптер: {}", response);
                    }
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
                            if repeat_flag_index.is_some() {
                                stream1 = repeat_request(stream1, input, repeat_time, String::new());
                            } else {
                                (stream1, response) = send_request(stream1, &input).unwrap();
                                println!("{}", response)
                            }
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
                    if repeat_flag_index.is_some() {
                        stream2 = repeat_request(stream2, 
                            input, 
                            repeat_time, 
                            String::from("Процент используемой физической памяти: "));
                    } else {
                        (stream2, response) = send_request(stream2, &input).expect("Ошибка записи в сокет");
                        println!("Процент используемой физической памяти: {}", response);
                    }
                }
            }, 
            "4" => {
                if is_stream2 {
                    if repeat_flag_index.is_some() {
                        stream2 = repeat_request(stream2, 
                            input, 
                            repeat_time, 
                            String::from("Процент используемой виртуальной памяти: "));
                    } else {
                        (stream2, response) = send_request(stream2, &input)
                        .expect("Ошибка записи в сокет");
                        println!("Процент используемой виртуальной памяти: {}", response)
                    }
                }
            },
            "q" => {
                return Result::Ok(());
            },
            _ => {
                
                // stream = send_request(stream, String::from("sdasdasdasd")).unwrap();
            }
        }




        // send_request(stream, input)?;

    }


}


fn send_request(stream_res: Result<TcpStream, io::Error>, request: &String) -> io::Result<(Result<TcpStream, io::Error>, String)> {
    let mut stream = stream_res.unwrap();
    stream.write_all(request.as_bytes()).expect("");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    return io::Result::Ok((Result::Ok(stream), response.into_owned()));

}

fn repeat_request(mut stream_res: Result<TcpStream, io::Error>, input: String, time: i32, label: String) -> Result<TcpStream, io::Error> {
    let stream = stream_res.unwrap();
    let mut stream_clone: Result<TcpStream, io::Error> = Result::Ok(stream.try_clone().unwrap());
    let (sender, receiver) = mpsc::channel::<i32>();
    
    let _ = thread::spawn(move || -> io::Result<()> {
        let mut response_th;
        loop {
            // Проверяем, не пришел ли сигнал остановки
            if receiver.try_recv().is_ok() {
                break;
            }
            
            (stream_clone, response_th) = send_request(stream_clone, &input).expect("Ошибка записи в сокет");
            if response_th != "null" {
                println!("{}{}", label, response_th);
            }

            thread::sleep(time::Duration::from_millis(time as u64));
        }
        Ok(())
    });
    stream_res = Result::Ok(stream);
    loop {
        let mut input = String::new();
        io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

        if input.trim() == "q" {
            sender.send(0).expect("ошибка отправки в канал");
            break;
        }
    }
    return stream_res;
}