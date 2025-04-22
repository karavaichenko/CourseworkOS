use std::{io::{Read, Write}, net::TcpStream, sync::mpsc};


pub struct Client {
    pub streams: [Option<TcpStream>; 2],
    server_addresses: [String; 2],
}

impl Client {
    pub fn new(server1_address: String, server2_address: String) -> Self {
        let stream1 = TcpStream::connect(server1_address.clone()).ok();
        let stream2 = TcpStream::connect(server2_address.clone()).ok();
        let streams = [stream1, stream2];
        let server_addresses = [server1_address, server2_address];
        return Self {streams, server_addresses};
    }

    pub fn disconnect(&mut self, server_num: i32) -> Option<()> {
        if 0 <= server_num && server_num < 2 {
            self.streams[server_num as usize] = None;
            Some(())
        } else {
            None
        }
    }

    pub fn connect(&mut self, server_num: i32) -> Result<(), ()> {
        if 0 <= server_num && server_num < 2 {
            self.streams[server_num as usize] = TcpStream::connect(self.server_addresses[server_num as usize].clone()).ok();
            if self.streams[server_num as usize].is_some() {Ok(())} else {Err(())}
        } else {
            return Err(());
        }
    }

    pub fn send_request(&mut self, server_num: i32, request: &String) -> Option<String> {
        let mut stream: TcpStream;
        if self.streams[server_num as usize].is_some() {
            stream = self.streams[server_num as usize].as_ref().unwrap().try_clone().unwrap();
        } else {
            if self.connect(server_num).is_ok() {
                return self.send_request(server_num, request);
            } else {
                return None;
            }
        }
        let write_result = stream.write_all(request.as_bytes());
        if write_result.is_err() {
            self.streams[server_num as usize] = None;
            return None;
        }
    
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer).expect("Не удалось прочитать из сокета");
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        
        return Some(response.into_owned());
    }

    pub fn repeated_requests(&mut self, server_num: i32, request: &String, time: i32) -> Option<String> {
        let request_cloned = request.clone();
        let stream: TcpStream;
        if 0 <= server_num && server_num < 2 {
            let stream_res = self.streams[server_num as usize].as_ref();
            if stream_res.is_some() {
                stream = stream_res.unwrap().try_clone().unwrap();
            } else {
                if self.connect(server_num).is_ok() {
                    return self.repeated_requests(server_num, request, time);
                } else {
                    return None;
                }
            }
        } else {
            return None;
        }
        let mut streams: [Option<TcpStream>; 2] = [None, None];
        streams[server_num as usize] = Some(stream);
        let mut client = Client { streams, server_addresses: ["".to_string(), "".to_string()] };
        let (sender, receiver) = mpsc::channel::<i32>();
        let _ = std::thread::spawn(move || -> std::io::Result<()> {
            let mut response_th;
            loop {
                // Проверяем, не пришел ли сигнал остановки
                if receiver.try_recv().is_ok() {
                    break;
                }
                
                response_th = client.send_request(server_num, &request_cloned).unwrap();
                let splited_resp: Vec<&str> = response_th.trim().split(" ").collect();
                let response = splited_resp.get(2).unwrap();
                if *response != "null" {
                    println!("{}", response_th);
                }

                std::thread::sleep(std::time::Duration::from_millis(time as u64));
            }
            Ok(())
        });
        loop {
            let mut input = String::new();
            std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
    
            if input.trim() == "q" {
                sender.send(0).expect("ошибка отправки в канал");
                break;
            }
        }
        Some("".to_string())
    }
}


