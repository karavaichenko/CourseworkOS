use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use server1::ThreadPool;
use std::process::Command;
#[cfg(target_os = "windows")]
use windows::{
    core::Result,
    Win32::{Graphics::Dxgi::*, System::Console::GetConsoleWindow, UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW}}
};

// Константы
const REPEAT_FLAG: &str = "-r";

// Функции под платформу
#[cfg(target_os = "linux")]
fn get_gpu_name() -> Result<Option<String>, ()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("lshw -C video | grep прод")
        .output()
        .map_err(|e| e.to_string())
        .expect("Ошибка получеия GPU");

    let gpu_info = String::from_utf8(output.stdout).map_err(|e| e.to_string()).expect("Ошибка получеия GPU");
    println!("{}", gpu_info);
    let res = Result::Ok(Option::Some(gpu_info.trim().to_string()));
    return res;
}

#[cfg(target_os = "windows")]
fn get_gpu_name() -> Result<Option<String>> {
    unsafe {
        // Создаём фабрику DXGI
        let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let adapter_index = 0;
        let mut gpu_name = None;

        // Перебираем доступные адаптеры
        loop {
            let adapter = factory.EnumAdapters(adapter_index);
            if adapter.is_err() {
                break; // Больше нет адаптеров
            }

            let adapter = adapter?;
            let desc = adapter.GetDesc()?;

            // Преобразуем широкий строковый буфер в String
            let name = String::from_utf16_lossy(&desc.Description);
            gpu_name = Some(name.trim_end_matches('\0').to_string());
            break; // Берём первую видеокарту
        }

        return Result::Ok(gpu_name);
    }
}

#[cfg(target_os = "linux")]
fn hide_console(time: i32) {
    let conn = Connection::new_session().expect("Ошибка сворачиваия ока");
    
    let prox = conn.with_proxy(
        "org.gnome.Shell",
        "/org/gnome/Shell",
        time::Duration::from_secs(2),
    );   

    let _: () = proxy.method_call("org.gnome.Shell", "MinimizeWindows", ()).expect("Ошибка сворачиваия ока");
}

#[cfg(target_os = "windows")]
fn hide_console(time: i32) {
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_invalid() {
            let _ = ShowWindow(hwnd, SW_HIDE);
            std::thread::sleep(std::time::Duration::from_millis(time as u64));
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }
}

enum Reuqest {
    PhysMemory,
    VirtualMemory
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();


        pool.execute(|| {
            handle_connection(stream);
        });
        

    }
}

fn handle_connection(mut stream: TcpStream) {

    let mut gpu_name_cache =  String::new();

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

        // Ищем флаги
        let repeat_flag_index = req_args.iter().position(|&x| x == REPEAT_FLAG);

        let arg1 = req_args.get(0).unwrap();
    
        // обработка запроса, получеие калла что адо отправить
        match arg1.trim() {
            "1" => {
                if repeat_flag_index.is_some() && gpu_name_cache.as_str() != "" {
                    let gpu_name = get_gpu_name().expect("Ошибка получения GPU").expect("Gpu не найден");
                    if gpu_name != gpu_name_cache {
                        gpu_name_cache = gpu_name.clone();
                        print!("{}", gpu_name);   
                        stream.write_all(gpu_name.as_bytes()).expect("ошибка записи в сокет");
                    }
                } else {
                    let gpu_name = get_gpu_name().expect("Ошибка получения GPU").expect("Gpu не найден");
                    gpu_name_cache = gpu_name.clone();
                    print!("{}", gpu_name);
                    stream.write_all(gpu_name.as_bytes()).expect("ошибка записи в сокет");
                }
            },
            "2" => {
                let time = req_args.get(1);
                // Проверка на наличие второго аргумента
                match time {
                    Some(time) => {
                        let time_int = time.trim().parse::<i32>();
                        // Проверка на число второго аргумента
                        match time_int {
                            Ok(time) => {
                                hide_console(time);
                                stream.write_all(b"1111").unwrap();
                            },
                            Err(_) => {
                                stream.write_all(b"invalid request").unwrap();
                                continue;
                            },
                        }
                    },
                    None => {
                        stream.write_all(b"invalid request").unwrap();
                        continue;
                    },
                }
            },
            _ => {
                println!("{}", arg1);
                stream.write_all(b"sosat").unwrap();
            }
        }
    }    
}
