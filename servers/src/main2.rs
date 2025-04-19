use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use chrono::Local;
use servers::{LogClient, ThreadPool};
use sysinfo::System;

// Константы
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

    let mut log_client = LogClient::new("log_server_2".to_string());

    let listener_res = TcpListener::bind("127.0.0.1:7979");
    let listener: TcpListener;
    match listener_res {
        Ok(lis) => {
            listener = lis;
        },
        Err(_) => {
            println!("Кажется сервер уже запущен, порт занят!");
            return;
        },
    }

    let pool = ThreadPool::new(10);

    let mut client_id = 0;
    for stream in listener.incoming() {
        log_client.write_log(&format!("CONNECT {} CLIENT", client_id));
        client_id += 1;
        let stream = stream.unwrap();
        let cloned_log_client = log_client.clone();


        pool.execute(move || {
            handle_connection(stream, cloned_log_client, client_id);
        });
    
    }
}

fn handle_connection(mut stream: TcpStream, mut log_client: LogClient, client_id: i32) {

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
                log_client.write_log(&format!("CLIENT {} DISCONNECT", client_id));
                break;
            },
        };
        let req: String = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        log_client.write_log(&format!("Server received from client {}: {}", client_id, req));
        // write_log(&mut pipe, &log_record);
        println!("Received: {}", req);
    
        let req_args: Vec<&str> = req.split(" ").collect();

        if req_args.len() < 1 {
            send_response(&mut stream, &"invalid request".to_string());
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
                        log_client.write_log(&format!("Server responded client {}: {}", client_id, phys_mem_str));
                    } else {
                        send_response(&mut stream, &"null".to_string());
                        log_client.write_log(&format!("Server responded client {}: null", client_id));
                    }
                } else {
                    let phys_mem = get_phys_mem().expect("ошибка физической памяти");
                    let phys_mem_str: String = phys_mem.to_string().chars().take(5).collect();
                    phys_mem_str_cache = phys_mem_str.clone();
                    send_response(&mut stream, &phys_mem_str);
                    log_client.write_log(&format!("Server responded client {}: {}", client_id, phys_mem_str));
                }
            },
            "4" => {
                if repeat_flag_index.is_some() && virtual_mem_str_cache.as_str() != "" {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    if virtual_mem_str != virtual_mem_str_cache {
                        virtual_mem_str_cache = virtual_mem_str.clone();
                        send_response(&mut stream, &virtual_mem_str);
                        log_client.write_log(&format!("Server responded client {}: {}", client_id, virtual_mem_str));
                    } else {
                        send_response(&mut stream, &"null".to_string());
                        log_client.write_log(&format!("Server responded client {}: null", client_id));
                    }
                } else {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    virtual_mem_str_cache = virtual_mem_str.clone();
                    send_response(&mut stream, &virtual_mem_str);
                    log_client.write_log(&format!("Server responded client {}: {}", client_id, virtual_mem_str));
                }
            },
            _ => {
                send_response(&mut stream, &"not found".to_string());
                log_client.write_log(&format!("Server responded client {}: not found", client_id));
            }
        }


    
        
    }    
}

fn send_response(stream: &mut TcpStream, response: &String) {
    let response_with_time = format!("{} {}", Local::now().format("%d.%m.%Y %T"), response);
    let _write_res = stream.write_all(response_with_time.as_bytes());
}