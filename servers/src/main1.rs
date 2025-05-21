use std::{
    io::prelude::*, net::{TcpListener, TcpStream}
};
use chrono::Local;
use servers::{LogClient, ThreadPool};
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use x11::xlib;
#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::ptr;
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
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            eprintln!("Не удалось подключиться к X11");
            return;
        }

        // Получаем Window ID текущего терминала
        let window_name = CString::new("").unwrap();
        let window = xlib::XDefaultRootWindow(display);
        let mut focus: xlib::Window = 0;
        xlib::XGetInputFocus(display, &mut focus, ptr::null_mut());

        // Сворачиваем окно терминала
        xlib::XIconifyWindow(display, focus, 0);
        xlib::XCloseDisplay(display);
    }
}

#[cfg(target_os = "windows")]
fn hide_console(time: u64) {
    use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_SHOW};
    use winapi::um::wincon::GetConsoleWindow;
    
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }
        
        ShowWindow(hwnd, SW_HIDE);
        
        winapi::um::winuser::UpdateWindow(hwnd);
        
        std::thread::sleep(std::time::Duration::from_millis(time));
        
        ShowWindow(hwnd, SW_SHOW);
        
        winapi::um::winuser::UpdateWindow(hwnd);
        winapi::um::winuser::SetForegroundWindow(hwnd);
    }
}

fn main() {

    let listener_res = TcpListener::bind("127.0.0.1:7878");
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

    // Подключение к лог серверу
    let mut log_client = LogClient::new("log_server_1".to_string());
    
    let pool = ThreadPool::new(10);
    
    let mut client_id = 0;
    for stream in listener.incoming() {
        client_id += 1;
        log_client.write_log(&format!("CONNECT {} CLIENT", client_id));
        let stream = stream.unwrap();
        let cloned_log_client = log_client.clone();

        pool.execute(move || {
            handle_connection(stream, cloned_log_client, client_id);
        });
        

    }
}

fn handle_connection(mut stream: TcpStream, mut log_client: LogClient, client_id: i32) {

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
                log_client.write_log(&format!("CLIENT {} DISCONNECT", client_id));
                break;
            },
        };

        let req: String = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
        log_client.write_log(&format!("Server received from client {}: {}", client_id, req));
    
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
                        send_response(&mut stream, &gpu_name);
                        log_client.write_log(&format!("Server responded client {}: {}", client_id, gpu_name));
                    } else {
                        send_response(&mut stream, &"null".to_string());
                        log_client.write_log(&format!("Server responded client {}: null", client_id));
                    }
                } else {
                    let gpu_name = get_gpu_name().expect("Ошибка получения GPU").expect("Gpu не найден");
                    gpu_name_cache = gpu_name.clone();
                    send_response(&mut stream, &gpu_name);
                    log_client.write_log(&format!("Server responded client {}: {}", client_id, gpu_name));
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
                                hide_console(time as u64);
                                send_response(&mut stream, &"success".to_string());
                                log_client.write_log(&format!("Server responded client {}: success", client_id));
                            },
                            Err(_) => {
                                send_response(&mut stream, &"invalid request".to_string());
                                log_client.write_log(&format!("Server responded client {}: invalid request", client_id));
                                continue;
                            },
                        }
                    },
                    None => {
                        send_response(&mut stream, &"invalid request".to_string());
                        log_client.write_log(&format!("Server responded client {}: invalid request", client_id));
                        continue;
                    },
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
