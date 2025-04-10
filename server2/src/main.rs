use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream}, str::Bytes,
};
use server2::ThreadPool;
use sysinfo::System;

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

    loop {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).unwrap(); // Читаем сырые байты
        let req: String = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        println!("Received: {}", req);
    
    
        // обработка запроса, получеие калла что адо отправить
        match req.trim() {
            "3" => {
                let phys_mem = get_phys_mem().expect("ошибка физической памяти");
                let phys_mem_str = phys_mem.to_string();
                // let response = String::from("Процент используемой физ. памяти: ") + phys_mem_str.as_str();
                stream.write_all(phys_mem_str.as_bytes()).expect("ошибка записи в сокет");
            },
            "4" => {
                let virtual_mem = get_virtual_mem().expect("ошибка виртуальной памяти");
                let virtual_mem_str = virtual_mem.to_string();
                let response = String::from("Процент используемой виртуальной памяти: ") + virtual_mem_str.as_str();
                stream.write_all(response.as_bytes()).unwrap();
            },
            _ => {
                stream.write_all(b"sosat").unwrap();
            }
        }


    
        
    }    
}