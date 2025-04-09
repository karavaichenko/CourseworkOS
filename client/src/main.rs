use std::io::{self, Read, Write};
use std::net::TcpStream;

#[warn(dead_code)]
enum Reuqest {
    PhysMemory,
    VirtualMemory
}


fn main() -> io::Result<()> {
    // Подключаемся к серверу
    let mut stream1 = TcpStream::connect("127.0.0.1:7878")?;
    let mut stream2 = TcpStream::connect("127.0.0.1:7979")?;
    println!("Connected to server!");


    loop {

        print!("1 - название используемого видеоадаптера\n");
        print!("2 (100-1000) - скрыть окно сервера на * мс\n");
        print!("3 - прцент используемой физической памяти\n");
        print!("4 - прцент используемой виртуальной памяти\n");
    
        // ввод запроса от клиета
        let mut input = String::new();
        io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

        let args: Vec<&str> = input.split(" ").collect();
        if args.len() < 1 {
            println!("Недостаточно аргументов!");
            continue;
        }
        let arg1 = args.get(0).unwrap();
    
        match arg1.trim() {
            "1" => {
                stream1 = send_request(stream1, input).unwrap();
            },
            "2" => {
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
                        stream1 = send_request(stream1, input).unwrap();
                    },
                    Err(_) => {
                        println!("Аргумент должен быть числом!!!");
                        continue;
                    },
                }
            }
            "3" | "4" => {
                stream2 = send_request(stream2, input).expect("Ошибка записи в сокет")
            }
            _ => {
                
                // stream = send_request(stream, String::from("sdasdasdasd")).unwrap();
            }
        }




        // send_request(stream, input)?;

    }


}


fn send_request(mut stream: TcpStream, request: String) -> io::Result<TcpStream> {
    stream.write_all(request.as_bytes()).expect("");

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    println!("\n\n");
    println!("Server response: {}", response);
    println!("\n\n");

    return Result::Ok(stream);

}