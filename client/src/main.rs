use std::io::{self};
use std::env;
use client::Client;



fn main() -> io::Result<()> {

    let server1_addr = env::var("SERVER1_ADDRESS").unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let server2_addr = env::var("SERVER2_ADDRESS").unwrap_or_else(|_| "127.0.0.1:7979".to_string());
    // Подключаемся к серверу
    let mut client = Client::new(server1_addr, server2_addr);

    println!("Connected to server!");


    loop {
        print!("\n\n");
        if client.streams[0].is_some() {
            print!("1 - название используемого видеоадаптера\n");
            print!("2 (100-1000) - скрыть окно сервера на * мс\n");
        }
        if client.streams[1].is_some() {
            print!("3 - процент используемой физической памяти\n");
            print!("4 - процент используемой виртуальной памяти\n");
        }
        if client.streams[0].is_none() || client.streams[1].is_none() {
            print!("connect (1-2) - подключиться к серверу\n")
        }
        if client.streams[0].is_some() || client.streams[1].is_some() {
            print!("disconnect (1-2) - отключиться от сервера\n")
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
                if client.streams[0].is_some() {
                    if repeat_flag_index.is_some() {
                        client.repeated_requests(0, &input, repeat_time).unwrap();
                    } else {
                        response = client.send_request(0, &input);
                        print_response(response);
                    }
                }
            },
            "2" => {
                if client.streams[0].is_some() {
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
                                client.repeated_requests(0, &input, repeat_time);
                            } else {
                                response = client.send_request(0, &input);
                                print_response(response);
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
                if client.streams[1].is_some() {
                    if repeat_flag_index.is_some() {
                        client.repeated_requests(1, &input, repeat_time);
                    } else {
                        response = client.send_request(1, &input);
                        print_response(response);
                    }
                }
            }, 
            "4" => {
                if client.streams[1].is_some() {
                    if repeat_flag_index.is_some() {
                        client.repeated_requests(1, &input, repeat_time);
                    } else {
                        response = client.send_request(1, &input);
                        print_response(response);
                    }
                }
            },
            "connect" => {
                if args.len() < 2 {
                    println!("Недостаточно аргументов");
                    continue;
                }
                let server_num_res = args
                .get(1)
                .unwrap().trim()
                .parse::<i32>();
                let server_num: i32 = match server_num_res {
                    Ok(num) => num,
                    Err(_) => {
                        print!("Аргумент должен быть числом");
                        continue;
                    },
                };

                let res = client.connect(server_num - 1);
                if res.is_ok() {
                    println!("Соединение установлено!");
                } else {
                    println!("Не удалось подключиться к серверу");
                }
                
            },
            "disconnect" => {
                if args.len() < 2 {
                    println!("Недостаточно аргументов");
                    continue;
                }
                let server_num_res = args
                .get(1)
                .unwrap().trim()
                .parse::<i32>();
                let server_num: i32 = match server_num_res {
                    Ok(num) => num,
                    Err(_) => {
                        print!("Аргумент должен быть числом");
                        continue;
                    },
                };

                let res = client.disconnect(server_num - 1);
                if res.is_some() {
                    println!("Соединение разорвано!");
                } else {
                    println!("Нет сервера с таким номером!")
                }
            },
            "q" => {
                return Result::Ok(());
            },
            _ => {

            }
        }
    }
}

fn print_response(resp: Option<String>) {
    if resp.is_some() {
        println!("{}", resp.unwrap());
    } else {
        println!("Не удалось выполнить запрос, соединение разорвано");
    }
}