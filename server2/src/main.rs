use std::{
    fs::{File, OpenOptions},
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use chrono::Local;
use server2::ThreadPool;
use sysinfo::System;
use std::os::windows::fs::OpenOptionsExt;
use winapi::um::winbase::FILE_FLAG_OVERLAPPED;

const REPEAT_FLAG: &str = "-r";


fn get_phys_mem() -> Result<f64, ()> {
    // Инициализируем систему
    let mut system = System::new_all();
    
    // Обновляем информацию о системе
    system.refresh_all();
    
    // Получаем PID текущего процесса
    let current_pid = sysinfo::get_current_pid().unwrap();
    
    // Получаем информацию о текущем процессе
    if let Some(process) = system.process(current_pid) {
        // Получаем общий объем физической памяти системы
        let total_physical_memory = system.total_memory();
        // Получаем используемую физическую память процессом (в KB)
        let used_physical_memory = process.memory();
        
        // Рассчитываем процент используемой физической памяти
        let physical_memory_percent = if total_physical_memory > 0 {
            (used_physical_memory as f64 / total_physical_memory as f64) * 100.0
        } else {
            0.0
        };
        
        return Result::Ok(physical_memory_percent);
    } else {
        return Result::Err(());
    }
}

fn get_virtual_mem() -> Result<f64, ()> {
    // Инициализируем систему
    let mut system = System::new_all();
    
    // Обновляем информацию о системе
    system.refresh_all();
    
    // Получаем PID текущего процесса
    let current_pid = sysinfo::get_current_pid().unwrap();

        // Получаем информацию о текущем процессе
        if let Some(process) = system.process(current_pid) {
            
            // Получаем общий объем виртуальной памяти системы
            let total_virtual_memory = system.total_swap();
            // Получаем используемую виртуальную память процессом (в KB)
            let used_virtual_memory = process.virtual_memory();
            
            // Рассчитываем процент используемой виртуальной памяти
            let virtual_memory_percent = if total_virtual_memory > 0 {
                (used_virtual_memory as f64 / total_virtual_memory as f64) * 100.0
            } else {
                0.0
            };
            
            return Result::Ok(virtual_memory_percent);
        } else {
            return Result::Err(());
        }
}

fn main() {

    let log_pipe = connect_log_server();

    let listener = TcpListener::bind("127.0.0.1:7979").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let cloned_pipe = log_pipe.try_clone().expect("Не удалось скопировать pipe");

        pool.execute(|| {
            handle_connection(stream, cloned_pipe);
        });
    
    }
}

fn handle_connection(mut stream: TcpStream, mut pipe: File) {

    let mut phys_mem_str_cache = String::new();
    let mut virtual_mem_str_cache = String::new();

    loop {
        let mut buffer = [0; 1024];
        let bytes_read_res = stream.read(&mut buffer);
        let bytes_read ;
        match bytes_read_res {
            Ok(bytes) => {
                bytes_read = bytes;
            },
            Err(_) => {
                break;
            },
        };
        let req: String = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        let mut log_record = "Server 2 received: ".to_string() + req.as_str();
        // write_log(&mut pipe, &log_record);
        println!("Received: {}", req);
    
        let req_args: Vec<&str> = req.split(" ").collect();

        if req_args.len() < 1 {
            send_response(&mut stream, &"invalid request".to_string()).unwrap();
            continue;
        }
        // ищем флаги
        let repeat_flag_index = req_args.iter().position(|&x| x == REPEAT_FLAG);
        let arg1 = req_args.get(0).unwrap();
    
    
        // обработка запроса, получеие калла что адо отправить
        match arg1.trim() {
            "3" => {
                if repeat_flag_index.is_some() && phys_mem_str_cache.as_str() != "" {
                    let phys_mem = get_phys_mem().expect("ошибка физической памяти");
                    let phys_mem_str: String = phys_mem.to_string().chars().take(5).collect();
                    if phys_mem_str != phys_mem_str_cache {
                        phys_mem_str_cache = phys_mem_str.clone();
                        send_response(&mut stream, &phys_mem_str).expect("ошибка записи в сокет");
                        log_record += "Server responded: ";
                        log_record += phys_mem_str.as_str();
                        write_log(&mut pipe,&log_record);
                    } else {
                        send_response(&mut stream, &"null".to_string()).expect("ошибка записи в сокет");
                        log_record += "Server responded: null";
                        write_log(&mut pipe,&log_record);
                    }
                } else {
                    let phys_mem = get_phys_mem().expect("ошибка физической памяти");
                    let phys_mem_str: String = phys_mem.to_string().chars().take(5).collect();
                    phys_mem_str_cache = phys_mem_str.clone();
                    send_response(&mut stream, &phys_mem_str).expect("ошибка записи в сокет");
                    log_record += "Server responded: ";
                    log_record += phys_mem_str.as_str();
                    write_log(&mut pipe,&log_record);
                }
            },
            "4" => {
                if repeat_flag_index.is_some() && virtual_mem_str_cache.as_str() != "" {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    if virtual_mem_str != virtual_mem_str_cache {
                        virtual_mem_str_cache = virtual_mem_str.clone();
                        send_response(&mut stream, &virtual_mem_str).expect("ошибка записи в сокет");
                        log_record += "Server responded: ";
                        log_record += virtual_mem_str.as_str();
                        write_log(&mut pipe,&log_record);
                    } else {
                        send_response(&mut stream, &"null".to_string()).expect("ошибка записи в сокет");
                        log_record += "Server responded: null";
                        write_log(&mut pipe,&log_record);
                    }
                } else {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    virtual_mem_str_cache = virtual_mem_str.clone();
                    send_response(&mut stream, &virtual_mem_str).unwrap();
                    log_record += "Server responded: ";
                    log_record += virtual_mem_str.as_str();
                    write_log(&mut pipe,&log_record);
                }
            },
            _ => {
                send_response(&mut stream, &"not found".to_string()).unwrap();
                log_record += "Server responded: not found";
                write_log(&mut pipe, &log_record);
            }
        }


    
        
    }    
}

fn connect_log_server() -> File {
    let mut pipe: File;
    loop {
        let pipe_name = r"\\.\pipe\log_server_2";
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
        println!("Ожидается включение лог сервера!");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
    }
    pipe.write_all(b"SERVER IS ON").unwrap();
    println!("Соединение с лог сервером установлено!");
    return pipe;
}

fn write_log(pipe: &mut File, data: &String) {
    pipe.write_all(data.as_bytes()).expect("Не удалось записать лог");
}

fn send_response(stream: &mut TcpStream, response: &String) -> std::io::Result<()> {

    let response_with_time = format!("{} {}", Local::now().format("%d.%m.%Y %T"), response);
    stream.write_all(response_with_time.as_bytes())?;

    Ok(())
}