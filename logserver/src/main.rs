use std::{io, thread};
use std::os::windows::io::FromRawHandle;
use winapi::ctypes::c_void;
use winapi::um::winbase::{PIPE_ACCESS_DUPLEX, PIPE_TYPE_MESSAGE, PIPE_WAIT};
use winapi::um::namedpipeapi::CreateNamedPipeW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::{io::{Read, Write}, fs::{File, OpenOptions}, fs};
use chrono::Local;


fn main() -> std::io::Result<()> {

    // Файл логов и канал для первого сервера
    fs::create_dir_all("logs_server").expect("Невозможно создать папку");
    let log_file1 = OpenOptions::new()
        .create(true)    // Создаем если не существует
        .append(true)    // Добавляем в конец файла
        .write(true)     // Разрешаем запись
        .open("./logs_server/log1.txt").expect("Невозможно открыть файл");

    let (pipe_stream1, _pipe1) = create_pipe("log_server_1").expect("Не удалось создать канал");
    
    // Файл логов и канал для второго сервера
    fs::create_dir_all("logs_server2").expect("Невозможно создать папку");
    let log_file2 = OpenOptions::new()
        .create(true)    // Создаем если не существует
        .append(true)    // Добавляем в конец файла
        .write(true)     // Разрешаем запись
        .open("./logs_server/log2.txt").expect("Невозможно открыть файл");

    let (pipe_stream2, _pipe2) = create_pipe("log_server_2").expect("Не удалось создать канал");
    
    // Создаём 2 потока для обоаботки логов
    let thread1 = thread::spawn(move || log_writer(pipe_stream1, log_file1));
    let thread2 = thread::spawn(move || log_writer(pipe_stream2, log_file2));

    let _ = thread1.join();
    let _ = thread2.join();

    Ok(())
}

fn create_pipe(name: &str) -> Result<(File, *mut c_void), std::io::Error> {
    let pipe_name = format!(r"\\.\pipe\{}", name);
    let wide_name: Vec<u16> = OsStr::new(pipe_name.as_str()).encode_wide().chain(Some(0)).collect();
    unsafe {
        let pipe = CreateNamedPipeW(
            wide_name.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_MESSAGE | PIPE_WAIT,  // Используем константу PIPE_WAIT
            1,   // Максимальное количество клиентов
            4096, // Размер буфера
            4096,
            0,   // Таймаут по умолчанию
            std::ptr::null_mut()  // Атрибуты безопасности
        );
        
        if pipe == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        println!("Сервер: канал создан. Ожидание клиента...");
        
        // Ожидаем подключения клиента
        let connected = winapi::um::namedpipeapi::ConnectNamedPipe(pipe, std::ptr::null_mut());
        if connected == 0 {
            let err = std::io::Error::last_os_error();
            winapi::um::handleapi::CloseHandle(pipe);
            return Err(err);
        }

        println!("Сервер: клиент подключен!");
        
        let pipe_stream = std::fs::File::from_raw_handle(pipe as _);
        return Result::Ok((pipe_stream, pipe));
    }
}

fn _close_pipe(pipe: *mut c_void) {
    unsafe {
        winapi::um::handleapi::CloseHandle(pipe);
    }
}

fn log_writer(mut pipe_stream: File, mut log_file: File) -> Result<(), io::Error> {
    let mut buf = [0u8; 1024];

    loop {
        
        // Читаем данные
        let bytes_read = pipe_stream.read(&mut buf)?;

        let response: String = String::from_utf8_lossy(&buf[..bytes_read]).into_owned();
        if bytes_read != 0 {
            println!("\n{}", response);
        }

        // тут будем писать в лог файл всю эту чепуху
        write_log(&mut log_file, response);
    }
    // Ok(())
}

fn write_log(log_file: &mut File, response: String) {
    write!(log_file, "\n{} {}", Local::now().format("%d.%m.%Y %T"), response).unwrap();
}