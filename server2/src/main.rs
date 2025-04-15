use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream}, str::Bytes,
};
use server2::ThreadPool;
use sysinfo::System;

const REPEAT_FLAG: &str = "-r";

enum Reuqest {
    PhysMemory,
    VirtualMemory
}

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
        println!("Received: {}", req);
    
        let req_args: Vec<&str> = req.split(" ").collect();

        if req_args.len() < 1 {
            stream.write_all(b"invalid request").unwrap();
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
                        stream.write_all(phys_mem_str.as_bytes()).expect("ошибка записи в сокет");
                    } else {
                        stream.write_all(b"null").expect("ошибка записи в сокет");
                    }
                } else {
                    let phys_mem = get_phys_mem().expect("ошибка физической памяти");
                    let phys_mem_str: String = phys_mem.to_string().chars().take(5).collect();
                    phys_mem_str_cache = phys_mem_str.clone();
                    stream.write_all(phys_mem_str.as_bytes()).expect("ошибка записи в сокет");
                }
            },
            "4" => {
                if repeat_flag_index.is_some() && virtual_mem_str_cache.as_str() != "" {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    if virtual_mem_str != virtual_mem_str_cache {
                        virtual_mem_str_cache = virtual_mem_str.clone();
                        stream.write_all(virtual_mem_str.as_bytes()).expect("ошибка записи в сокет");
                    } else {
                        stream.write_all(b"null").expect("ошибка записи в сокет");
                    }
                } else {
                    let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                    let virtual_mem_str: String = virtual_mem.to_string().chars().take(5).collect();
                    virtual_mem_str_cache = virtual_mem_str.clone();
                    stream.write_all(virtual_mem_str.as_bytes()).unwrap();
                }
            },
            _ => {
                stream.write_all(b"sosat").unwrap();
            }
        }


    
        
    }    
}