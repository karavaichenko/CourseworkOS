use std::os::windows::io::{AsRawHandle, FromRawHandle};
use winapi::um::winbase::{PIPE_ACCESS_DUPLEX, PIPE_TYPE_MESSAGE, PIPE_WAIT};
use winapi::um::namedpipeapi::CreateNamedPipeW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let pipe_name = r"\\.\pipe\my_rust_pipe";
    let wide_name: Vec<u16> = OsStr::new(pipe_name).encode_wide().chain(Some(0)).collect();
    
    unsafe {
        // Создаем канал с явным указанием PIPE_WAIT
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
        
        let mut pipe_stream = std::fs::File::from_raw_handle(pipe as _);

        
        let mut buf = [0u8; 1024];
        loop {
            
            // Читаем данные
            let bytes_read = pipe_stream.read(&mut buf)?;

            if bytes_read != 0 {
                println!("Сервер получил: {}", String::from_utf8_lossy(&buf[..bytes_read]));
            }

            // тут будем писать в лог файл всю эту чепуху
        }
        // Закрываем канал
        winapi::um::handleapi::CloseHandle(pipe);
    }
    Ok(())
}